#pragma once

#include <windows.h>
#include "AutoclickerEngine.h"

class MainWindow {
public:
    MainWindow(HINSTANCE hInstance);
    ~MainWindow();

    bool Create();
    void Show(int nCmdShow);

private:
    static LRESULT CALLBACK StaticWndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);
    LRESULT HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);

    void UpdateLanguage();
    void UpdateControlStates();
    void StartAutoclicker();
    void StopAutoclicker();

    HINSTANCE hInst;
    HWND m_hWnd = nullptr;
    AutoclickerEngine engine;

    // UI Controls
    HWND hEditClickRate, hLabelClickRate;
    HWND hLabelClickCount, hEditClickCount;
    HWND hEditStartButton, hEditStopButton;
    HWND hComboClickType, hLabelButtonType;
    HWND hRadioUntilStopped, hRadioUntilCount;
    HWND hButtonLang;
    HWND hEditSetKeybind, hLabelKeybind;
    HWND hGroupSettings, hGroupStop, hGroupHotkey;

    HFONT hFontModern = nullptr;
    HFONT hFontBigButton = nullptr;

    bool isSpanish = true;
    LRESULT currentHotkey = 0;
};
