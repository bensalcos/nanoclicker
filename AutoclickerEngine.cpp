#include "AutoclickerEngine.h"
#include <timeapi.h>

#pragma comment(lib, "winmm.lib")

AutoclickerEngine::AutoclickerEngine() {
    timeBeginPeriod(1); // Resolución del timer a 1ms
    clickerThread = std::thread(&AutoclickerEngine::ClickerTask, this);
}

AutoclickerEngine::~AutoclickerEngine() {
    shouldExit = true;
    clickerCV.notify_all();
    if (clickerThread.joinable()) {
        clickerThread.join();
    }
    timeEndPeriod(1); // Restaurar timer
}

void AutoclickerEngine::SetOnStopCallback(std::function<void()> callback) {
    onStopCallback = callback;
}

bool AutoclickerEngine::IsRunning() const {
    return isRunning.load();
}

void AutoclickerEngine::Start(int rate, int max, int type, bool infinite) {
    if (isRunning) return;

    clickRate = rate;
    maxClicks = max;
    clickType = type;
    infiniteMode = infinite;
    
    clicksDone = 0;
    isRunning = true;
    
    clickerCV.notify_all();
}

void AutoclickerEngine::Stop() {
    if (!isRunning) return;
    isRunning = false;
}

void AutoclickerEngine::ClickerTask() {
    while (!shouldExit) {
        std::unique_lock<std::mutex> lock(clickerMutex);
        clickerCV.wait(lock, [this]{ return isRunning.load() || shouldExit.load(); });
        
        if (shouldExit) break;
        
        lock.unlock();

        INPUT input = {0};
        input.type = INPUT_MOUSE;
        
        DWORD downFlag = MOUSEEVENTF_LEFTDOWN;
        DWORD upFlag = MOUSEEVENTF_LEFTUP;

        if (clickType == 1) { downFlag = MOUSEEVENTF_RIGHTDOWN; upFlag = MOUSEEVENTF_RIGHTUP; }
        else if (clickType == 2) { downFlag = MOUSEEVENTF_MIDDLEDOWN; upFlag = MOUSEEVENTF_MIDDLEUP; }

        input.mi.dwFlags = downFlag;
        SendInput(1, &input, sizeof(INPUT));
        
        input.mi.dwFlags = upFlag;
        SendInput(1, &input, sizeof(INPUT));

        if (!infiniteMode) {
            clicksDone++;
            if (clicksDone >= maxClicks) {
                isRunning = false;
                if (onStopCallback) {
                    onStopCallback();
                }
            }
        }
        
        if (isRunning) {
            Sleep(clickRate);
        }
    }
}
