#include <string>
#include <fstream>
#include <stdlib.h>
#include <algorithm>
#include "../DirTree.hh"
#include "../Event.hh"
#include "./BSER.hh"
#include "./watchman.hh"

#ifdef _WIN32
#define S_ISDIR(mode) ((mode & _S_IFDIR) == _S_IFDIR)
#define popen _popen
#define pclose _pclose
#endif

template<typename T>
BSER readBSER(T &&do_read) {
  std::stringstream oss;
  char buffer[256];
  int r;
  int64_t len = -1;
  do {
    // Start by reading a minimal amount of data in order to decode the length.
    // After that, attempt to read the remaining length, up to the buffer size.
    r = do_read(buffer, len == -1 ? 20 : (len < 256 ? len : 256));
    oss << std::string(buffer, r);

    if (len == -1) {
      uint64_t l = BSER::decodeLength(oss);
      len = l + oss.tellg();
    }

    len -= r;
  } while (len > 0);

  return BSER(oss);
}

std::string getSockPath() {
  printf("get sock\n");
  fflush(stdout);
  auto var = getenv("WATCHMAN_SOCK");
  if (var && *var) {
    return std::string(var);
  }

  printf("popen\n");
  fflush(stdout);
  FILE *fp = popen("watchman --output-encoding=bser get-sockname", "r");
  if (fp == NULL || errno == ECHILD) {
    printf("error exec\n");
    fflush(stdout);
    throw "Failed to execute watchman";
  }

  printf("read\n");
  fflush(stdout);
  BSER b = readBSER([fp] (char *buf, size_t len) {
    return fread(buf, sizeof(char), len, fp);
  });

  pclose(fp);
  printf("here\n");
  fflush(stdout);
  return b.objectValue().find("sockname")->second.stringValue();
}

std::unique_ptr<IPC> watchmanConnect() {
  std::string path = getSockPath();
  printf("%s\n", path.c_str());
  fflush(stdout);
  return std::unique_ptr<IPC>(new IPC(path));
}

BSER watchmanRead(IPC *ipc) {
  return readBSER([ipc] (char *buf, size_t len) {
    return ipc->read(buf, len);
  });
}

BSER::Object WatchmanBackend::watchmanRequest(BSER b) {
  std::string cmd = b.encode();
  mIPC->write(cmd);

  mRequestSignal.notify();
  mResponseSignal.wait();
  mResponseSignal.reset();
  return mResponse;
}

void WatchmanBackend::watchmanWatch(std::string dir) {
  std::vector<BSER> cmd;
  cmd.push_back("watch");
  cmd.push_back(dir);
  watchmanRequest(cmd);
}

bool WatchmanBackend::checkAvailable() {
  printf("check\n");
  fflush(stdout);
  try {
    watchmanConnect();
    printf("after connect\n");
    fflush(stdout);
    return true;
  } catch (const char *err) {
    printf("%s\n", err);
    fflush(stdout);
    return false;
  }
}

void handleFiles(Watcher &watcher, BSER::Object obj) {
  auto found = obj.find("files");
  if (found == obj.end()) {
    throw "Error reading changes from watchman";
  }
  
  auto files = found->second.arrayValue();
  for (auto it = files.begin(); it != files.end(); it++) {
    auto file = it->objectValue();
    auto name = file.find("name")->second.stringValue();
    #ifdef _WIN32
      std::replace(name.begin(), name.end(), '/', '\\');
    #endif
    auto mode = file.find("mode")->second.intValue();
    auto isNew = file.find("new")->second.boolValue();
    auto exists = file.find("exists")->second.boolValue();
    auto path = watcher.mDir + DIR_SEP + name;
    if (watcher.isIgnored(path)) {
      continue;
    }

    if (isNew && exists) {
      watcher.mEvents.create(path);
    } else if (exists && !S_ISDIR(mode)) {
      watcher.mEvents.update(path);
    } else if (!isNew && !exists) {
      watcher.mEvents.remove(path);
    }
  }
}

void WatchmanBackend::handleSubscription(BSER::Object obj) {
  std::unique_lock<std::mutex> lock(mMutex);
  auto subscription = obj.find("subscription")->second.stringValue();
  auto it = mSubscriptions.find(subscription);
  if (it == mSubscriptions.end()) {
    return;
  }

  auto watcher = it->second;
  handleFiles(*watcher, obj);
  watcher->notify();
}

void WatchmanBackend::start() {
  printf("start\n");
  fflush(stdout);
  mIPC = watchmanConnect();
  printf("got ipc\n");
  fflush(stdout);
  notifyStarted();

  while (true) {
    // If there are no subscriptions we are reading, wait for a request.
    if (mSubscriptions.size() == 0) {
      mRequestSignal.wait();
      mRequestSignal.reset();
    }

    // Break out of loop if we are stopped.
    if (mStopped) {
      break;
    }

    // Attempt to read from the socket.
    // If there is an error and we are stopped, break.
    BSER b;
    try {
      b = watchmanRead(&*mIPC);
    } catch (const char *err) {
      if (mStopped) {
        break;
      } else {
        throw err;
      }
    }

    auto obj = b.objectValue();
    auto error = obj.find("error");
    if (error != obj.end()) {
      throw error->second.stringValue().c_str();
    }

    // If this message is for a subscription, handle it, otherwise notify the request.
    auto subscription = obj.find("subscription");
    if (subscription != obj.end()) {
      handleSubscription(obj);
    } else {
      mResponse = obj;
      mResponseSignal.notify();
    }
  }

  mEndedSignal.notify();
}

WatchmanBackend::~WatchmanBackend() {
  std::unique_lock<std::mutex> lock(mMutex);

  // Mark the watcher as stopped, close the socket, and trigger the lock.
  // This will cause the read loop to be broken and the thread to exit.
  mStopped = true;
  mIPC.reset();
  mRequestSignal.notify();

  // If not ended yet, wait.
  mEndedSignal.wait();
}

std::string WatchmanBackend::clock(Watcher &watcher) {
  BSER::Array cmd;
  cmd.push_back("clock");
  cmd.push_back(watcher.mDir);

  BSER::Object obj = watchmanRequest(cmd);
  auto found = obj.find("clock");
  if (found == obj.end()) {
    throw "Error reading clock from watchman";
  }

  return found->second.stringValue();
}

void WatchmanBackend::writeSnapshot(Watcher &watcher, std::string *snapshotPath) {
  std::unique_lock<std::mutex> lock(mMutex);
  printf("writing snapshot\n");
  watchmanWatch(watcher.mDir);

  std::ofstream ofs(*snapshotPath);
  ofs << clock(watcher);
}

void WatchmanBackend::getEventsSince(Watcher &watcher, std::string *snapshotPath) {
  std::unique_lock<std::mutex> lock(mMutex);
  std::ifstream ifs(*snapshotPath);
  if (ifs.fail()) {
    return;
  }

  watchmanWatch(watcher.mDir);

  std::string clock;
  ifs >> clock;

  BSER::Array cmd;
  cmd.push_back("since");
  cmd.push_back(watcher.mDir);
  cmd.push_back(clock);

  BSER::Object obj = watchmanRequest(cmd);
  handleFiles(watcher, obj);
}

std::string getId(Watcher &watcher) {
  std::ostringstream id;
  id << "fschanges-";
  id << (void *)&watcher;
  return id.str();
}

void WatchmanBackend::subscribe(Watcher &watcher) {
  printf("subscribe watchman\n");
  fflush(stdout);
  watchmanWatch(watcher.mDir);
  printf("watched\n");
  fflush(stdout);

  std::string id = getId(watcher);
  mSubscriptions.emplace(id, &watcher);

  printf("got id\n");
  fflush(stdout);

  BSER::Array cmd;
  cmd.push_back("subscribe");
  cmd.push_back(watcher.mDir);
  cmd.push_back(id);

  BSER::Array fields;
  fields.push_back("name");
  fields.push_back("mode");
  fields.push_back("exists");
  fields.push_back("new");

  BSER::Object opts;
  opts.emplace("fields", fields);
  opts.emplace("since", clock(watcher));

  if (watcher.mIgnore.size() > 0) {
    BSER::Array ignore;
    BSER::Array anyOf;
    anyOf.push_back("anyof");

    for (auto it = watcher.mIgnore.begin(); it != watcher.mIgnore.end(); it++) {
      std::string pathStart = watcher.mDir + DIR_SEP;
      if (it->rfind(pathStart, 0) == 0) {
        auto relative = it->substr(pathStart.size());
        BSER::Array dirname;
        dirname.push_back("dirname");
        dirname.push_back(relative);
        anyOf.push_back(dirname);
      }
    }

    ignore.push_back("not");
    ignore.push_back(anyOf);

    opts.emplace("expression", ignore);
  }

  cmd.push_back(opts);
  printf("subscribing...\n");
  fflush(stdout);
  watchmanRequest(cmd);
}

void WatchmanBackend::unsubscribe(Watcher &watcher) {
  std::string id = getId(watcher);
  auto erased = mSubscriptions.erase(id);
  
  if (erased) {
    BSER::Array cmd;
    cmd.push_back("unsubscribe");
    cmd.push_back(watcher.mDir);
    cmd.push_back(id);

    watchmanRequest(cmd);
  }
}
