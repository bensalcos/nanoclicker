#pragma once

#include <windows.h>
#include <thread>
#include <atomic>
#include <mutex>
#include <condition_variable>
#include <functional>

class AutoclickerEngine {
public:
    AutoclickerEngine();
    ~AutoclickerEngine();

    void Start(int rate, int max, int type, bool infinite);
    void Stop();

    void SetOnStopCallback(std::function<void()> callback);

    bool IsRunning() const;

private:
    void ClickerTask();

    std::atomic<bool> isRunning{false};
    std::atomic<bool> shouldExit{false};
    std::thread clickerThread;
    std::mutex clickerMutex;
    std::condition_variable clickerCV;

    int clickRate = 100;
    int maxClicks = 0;
    int clicksDone = 0;
    bool infiniteMode = true;
    int clickType = 0; // 0 = Izquierdo, 1 = Derecho, 2 = Central

    std::function<void()> onStopCallback = nullptr;
};
