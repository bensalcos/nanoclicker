#include "MainWindow.h"
#include "Resource.h"
#include <commctrl.h>

#pragma comment(lib, "comctl32.lib")

#define MIN_WINDOW_WIDTH 420
#define MIN_WINDOW_HEIGHT 540
#define HOTKEY_ID 1
#define TIMER_HOTKEY 1

enum {
    ID_RADIO_INFINITE = 8000,
    ID_RADIO_COUNT = 8001,
    ID_COMBO_CLICKTYPE = 8002,
    ID_EDIT_CLICKRATE = 8003,
    ID_LABEL_CLICKRATE = 8004,
    ID_EDIT_CLICKCOUNT = 8005,
    ID_LABEL_CLICKCOUNT = 8006,
    ID_BUTTON_START = 8007,
    ID_BUTTON_STOP = 8008,
    ID_HOTKEY_BIND = 8009,
    ID_LABEL_HOTKEY = 8010,
    ID_BUTTON_LANG = 8011,
    ID_GROUP_SETTINGS = 8012,
    ID_GROUP_STOP = 8013,
    ID_GROUP_HOTKEY = 8014,
    ID_LABEL_BUTTONTYPE = 8015
};

static void SetFontToChildren(HWND hwnd, HFONT hFont) {
    EnumChildWindows(hwnd, [](HWND hwndChild, LPARAM lParam) -> BOOL {
        SendMessage(hwndChild, WM_SETFONT, (WPARAM)lParam, TRUE);
        return TRUE;
    }, (LPARAM)hFont);
}

MainWindow::MainWindow(HINSTANCE hInstance) : hInst(hInstance) {
    engine.SetOnStopCallback([this]() {
        PostMessage(m_hWnd, WM_COMMAND, ID_BUTTON_STOP, 0);
    });
}

MainWindow::~MainWindow() {
}

bool MainWindow::Create() {
    WNDCLASSEXW wcex = { sizeof(WNDCLASSEX) };
    wcex.style          = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc    = MainWindow::StaticWndProc;
    wcex.cbClsExtra     = 0;
    wcex.cbWndExtra     = 0;
    wcex.hInstance      = hInst;
    wcex.hIcon          = LoadIcon(hInst, MAKEINTRESOURCE(IDI_AUTOCLICKER));
    wcex.hCursor        = LoadCursor(nullptr, IDC_ARROW);
    wcex.hbrBackground  = CreateSolidBrush(RGB(255, 255, 255)); // Fondo blanco
    wcex.lpszMenuName   = MAKEINTRESOURCEW(IDC_AUTOCLICKER);
    wcex.lpszClassName  = L"AutoclickerClass";
    wcex.hIconSm        = LoadIcon(hInst, MAKEINTRESOURCE(IDI_SMALL));

    RegisterClassExW(&wcex);

    m_hWnd = CreateWindowW(L"AutoclickerClass", L"NanoClick", WS_OVERLAPPEDWINDOW,
      CW_USEDEFAULT, 0, MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT, nullptr, nullptr, hInst, this);

    return m_hWnd != nullptr;
}

void MainWindow::Show(int nCmdShow) {
    ShowWindow(m_hWnd, nCmdShow);
    UpdateWindow(m_hWnd);
}

LRESULT CALLBACK MainWindow::StaticWndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    MainWindow* pThis = nullptr;
    if (message == WM_NCCREATE) {
        CREATESTRUCT* pCreate = (CREATESTRUCT*)lParam;
        pThis = (MainWindow*)pCreate->lpCreateParams;
        SetWindowLongPtr(hWnd, GWLP_USERDATA, (LONG_PTR)pThis);
        pThis->m_hWnd = hWnd;
    } else {
        pThis = (MainWindow*)GetWindowLongPtr(hWnd, GWLP_USERDATA);
    }

    if (pThis) {
        return pThis->HandleMessage(hWnd, message, wParam, lParam);
    }

    return DefWindowProc(hWnd, message, wParam, lParam);
}

LRESULT MainWindow::HandleMessage(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    switch (message)
    {
        case WM_CREATE: 
        {
            hFontModern = CreateFontW(17, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET, 
                                      OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 
                                      DEFAULT_PITCH | FF_SWISS, L"Segoe UI");

            hFontBigButton = CreateFontW(20, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE, DEFAULT_CHARSET, 
                                      OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 
                                      DEFAULT_PITCH | FF_SWISS, L"Segoe UI");

            hButtonLang = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON | BS_FLAT, 0, 0, 0, 0, hWnd, (HMENU)ID_BUTTON_LANG, hInst, NULL);

            hGroupSettings = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_GROUPBOX, 0, 0, 0, 0, hWnd, NULL, hInst, NULL);
            hGroupStop = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_GROUPBOX, 0, 0, 0, 0, hWnd, NULL, hInst, NULL);
            hGroupHotkey = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_GROUPBOX, 0, 0, 0, 0, hWnd, NULL, hInst, NULL);

            hLabelClickRate = CreateWindowW(L"STATIC", L"", WS_VISIBLE | WS_CHILD, 0, 0, 0, 0, hWnd, (HMENU)ID_LABEL_CLICKRATE, hInst, NULL);
            hEditClickRate = CreateWindowW(L"EDIT", L"100", WS_VISIBLE | WS_CHILD | WS_BORDER | ES_NUMBER | ES_CENTER, 0, 0, 0, 0, hWnd, (HMENU)ID_EDIT_CLICKRATE, hInst, NULL);
            SendMessage(hEditClickRate, EM_LIMITTEXT, 6, 0);
            
            hLabelButtonType = CreateWindowW(L"STATIC", L"", WS_VISIBLE | WS_CHILD, 0, 0, 0, 0, hWnd, (HMENU)ID_LABEL_BUTTONTYPE, hInst, NULL);
            hComboClickType = CreateWindowW(L"COMBOBOX", NULL, WS_VISIBLE | WS_CHILD | CBS_DROPDOWNLIST | WS_VSCROLL, 0, 0, 0, 0, hWnd, (HMENU)ID_COMBO_CLICKTYPE, hInst, NULL);

            hRadioUntilStopped = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_AUTORADIOBUTTON | WS_GROUP, 0, 0, 0, 0, hWnd, (HMENU)ID_RADIO_INFINITE, hInst, NULL);
            SendMessage(hRadioUntilStopped, BM_SETCHECK, BST_CHECKED, 0);

            hRadioUntilCount = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_AUTORADIOBUTTON, 0, 0, 0, 0, hWnd, (HMENU)ID_RADIO_COUNT, hInst, NULL);
            hEditClickCount = CreateWindowW(L"EDIT", L"10", WS_VISIBLE | WS_CHILD | WS_BORDER | ES_NUMBER | ES_CENTER, 0, 0, 0, 0, hWnd, (HMENU)ID_EDIT_CLICKCOUNT, hInst, NULL);
            SendMessage(hEditClickCount, EM_LIMITTEXT, 6, 0);

            hLabelKeybind = CreateWindowW(L"STATIC", L"", WS_VISIBLE | WS_CHILD, 0, 0, 0, 0, hWnd, (HMENU)ID_LABEL_HOTKEY, hInst, NULL);
            hEditSetKeybind = CreateWindowExW(0, HOTKEY_CLASS, L"", WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP, 0, 0, 0, 0, hWnd, (HMENU)ID_HOTKEY_BIND, hInst, NULL);
            
            // Atajo por defecto: F6
            currentHotkey = MAKEWORD(VK_F6, 0);
            SendMessage(hEditSetKeybind, HKM_SETHOTKEY, currentHotkey, 0);
            RegisterHotKey(hWnd, HOTKEY_ID, 0, VK_F6);
            SetTimer(hWnd, TIMER_HOTKEY, 500, NULL);

            hEditStartButton = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, 0, 0, 0, 0, hWnd, (HMENU)ID_BUTTON_START, hInst, NULL);
            hEditStopButton = CreateWindowW(L"BUTTON", L"", WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, 0, 0, 0, 0, hWnd, (HMENU)ID_BUTTON_STOP, hInst, NULL);
            
            SetFontToChildren(hWnd, hFontModern);
            SendMessage(hEditStartButton, WM_SETFONT, (WPARAM)hFontBigButton, TRUE);
            SendMessage(hEditStopButton, WM_SETFONT, (WPARAM)hFontBigButton, TRUE);
            
            UpdateLanguage();
            UpdateControlStates();
        }
        break;

        case WM_SIZE:
        {
            int width = LOWORD(lParam);
            int height = HIWORD(lParam);
            int margin = 20;
            int groupWidth = width - (margin * 2);

            MoveWindow(hButtonLang, width - 60, 10, 40, 30, TRUE);

            // Grupo 1: Configuración
            int g1y = 45;
            MoveWindow(hGroupSettings, margin, g1y, groupWidth, 110, TRUE);
            MoveWindow(hLabelClickRate, margin + 20, g1y + 30, 200, 25, TRUE);
            MoveWindow(hEditClickRate, margin + 230, g1y + 30, 100, 25, TRUE);
            MoveWindow(hLabelButtonType, margin + 20, g1y + 70, 200, 25, TRUE);
            MoveWindow(hComboClickType, margin + 230, g1y + 70, 100, 100, TRUE);

            // Grupo 2: Parada
            int g2y = 170;
            MoveWindow(hGroupStop, margin, g2y, groupWidth, 100, TRUE);
            MoveWindow(hRadioUntilStopped, margin + 20, g2y + 30, 320, 25, TRUE);
            MoveWindow(hRadioUntilCount, margin + 20, g2y + 60, 220, 25, TRUE);
            MoveWindow(hEditClickCount, margin + 250, g2y + 60, 80, 25, TRUE);

            // Grupo 3: Atajo
            int g3y = 285;
            MoveWindow(hGroupHotkey, margin, g3y, groupWidth, 75, TRUE);
            MoveWindow(hLabelKeybind, margin + 20, g3y + 30, 200, 25, TRUE);
            MoveWindow(hEditSetKeybind, margin + 230, g3y + 30, 100, 25, TRUE);

            // Botones
            int btnWidth = 140;
            int btnHeight = 45;
            int btnY = height - margin - btnHeight;
            int center = width / 2;
            MoveWindow(hEditStartButton, center - btnWidth - 10, btnY, btnWidth, btnHeight, TRUE);
            MoveWindow(hEditStopButton, center + 10, btnY, btnWidth, btnHeight, TRUE);
        }
        break;

        case WM_CTLCOLORSTATIC:
        {
            HDC hdcStatic = (HDC)wParam;
            SetBkColor(hdcStatic, RGB(255, 255, 255));
            SetBkMode(hdcStatic, OPAQUE);
            return (LRESULT)GetStockObject(WHITE_BRUSH);
        }

        case WM_TIMER:
        {
            if (wParam == TIMER_HOTKEY) {
                LRESULT hk = SendMessage(hEditSetKeybind, HKM_GETHOTKEY, 0, 0);
                if (hk != currentHotkey && hk != 0) {
                    UnregisterHotKey(hWnd, HOTKEY_ID);
                    currentHotkey = hk;
                    UINT wVirtualKeyCode = LOBYTE(LOWORD(hk));
                    UINT wModifiers = HIBYTE(LOWORD(hk));
                    UINT fsModifiers = 0;
                    if (wModifiers & HOTKEYF_ALT) fsModifiers |= MOD_ALT;
                    if (wModifiers & HOTKEYF_CONTROL) fsModifiers |= MOD_CONTROL;
                    if (wModifiers & HOTKEYF_SHIFT) fsModifiers |= MOD_SHIFT;
                    RegisterHotKey(hWnd, HOTKEY_ID, fsModifiers, wVirtualKeyCode);
                }
            }
        }
        break;

        case WM_HOTKEY:
        {
            if (wParam == HOTKEY_ID) {
                if (engine.IsRunning()) StopAutoclicker();
                else StartAutoclicker();
            }
        }
        break;

        case WM_COMMAND:
        {
            int wmId = LOWORD(wParam);
            
            if (wmId == ID_RADIO_INFINITE || wmId == ID_RADIO_COUNT) {
                UpdateControlStates();
            }

            switch (wmId)
            {
            case ID_BUTTON_START:
                StartAutoclicker();
                break;
            case ID_BUTTON_STOP:
                StopAutoclicker();
                break;
            case ID_BUTTON_LANG:
                isSpanish = !isSpanish;
                UpdateLanguage();
                break;
            case IDM_EXIT:
                DestroyWindow(hWnd);
                break;
            default:
                return DefWindowProc(hWnd, message, wParam, lParam);
            }
        }
        break;

        case WM_GETMINMAXINFO:
        {
            LPMINMAXINFO lpMMI = (LPMINMAXINFO)lParam;
            lpMMI->ptMinTrackSize.x = MIN_WINDOW_WIDTH; 
            lpMMI->ptMinTrackSize.y = MIN_WINDOW_HEIGHT;
            return 0;
        }

        case WM_DESTROY:
            KillTimer(hWnd, TIMER_HOTKEY);
            StopAutoclicker();
            UnregisterHotKey(hWnd, HOTKEY_ID);
            if (hFontModern) DeleteObject(hFontModern);
            if (hFontBigButton) DeleteObject(hFontBigButton);
            PostQuitMessage(0);
            break;

        default:
            return DefWindowProc(hWnd, message, wParam, lParam);
    }
    return 0;
}

void MainWindow::StartAutoclicker() {
    if (engine.IsRunning()) return;

    WCHAR buffer[256];
    GetWindowTextW(hEditClickRate, buffer, 256);
    int clickRate = _wtoi(buffer);
    if (clickRate < 1) clickRate = 1;

    GetWindowTextW(hEditClickCount, buffer, 256);
    int maxClicks = _wtoi(buffer);

    bool infiniteMode = (SendMessage(hRadioUntilStopped, BM_GETCHECK, 0, 0) == BST_CHECKED);
    int clickType = static_cast<int>(SendMessage(hComboClickType, CB_GETCURSEL, 0, 0));

    engine.Start(clickRate, maxClicks, clickType, infiniteMode);
    UpdateControlStates();
}

void MainWindow::StopAutoclicker() {
    if (!engine.IsRunning()) return;
    engine.Stop();
    UpdateControlStates();
}

void MainWindow::UpdateControlStates() {
    bool isRunning = engine.IsRunning();
    bool isCustomCount = (SendMessage(hRadioUntilCount, BM_GETCHECK, 0, 0) == BST_CHECKED);
    
    EnableWindow(hEditClickCount, isCustomCount && !isRunning);
    EnableWindow(hEditClickRate, !isRunning);
    EnableWindow(hRadioUntilStopped, !isRunning);
    EnableWindow(hRadioUntilCount, !isRunning);
    EnableWindow(hComboClickType, !isRunning);
    EnableWindow(hEditSetKeybind, !isRunning);

    EnableWindow(hEditStartButton, !isRunning);
    EnableWindow(hEditStopButton, isRunning);
}

void MainWindow::UpdateLanguage() {
    int clickType = static_cast<int>(SendMessage(hComboClickType, CB_GETCURSEL, 0, 0));

    if (isSpanish) {
        SetWindowTextW(hGroupSettings, L" Configuraci\x00f3n ");
        SetWindowTextW(hGroupStop, L" Condici\x00f3n de Parada ");
        SetWindowTextW(hGroupHotkey, L" Atajo Global ");
        SetWindowTextW(hLabelClickRate, L"Intervalo de Clics (ms):");
        SetWindowTextW(hLabelButtonType, L"Bot\x00f3n del Mouse:");
        SetWindowTextW(hRadioUntilStopped, L"Hacer clic hasta detener");
        SetWindowTextW(hRadioUntilCount, L"Detener al llegar a (clics):");
        SetWindowTextW(hLabelKeybind, L"Presiona para Iniciar/Parar:");
        SetWindowTextW(hEditStartButton, L"INICIAR");
        SetWindowTextW(hEditStopButton, L"DETENER");
        SetWindowTextW(hButtonLang, L"EN");
        
        SendMessage(hComboClickType, CB_RESETCONTENT, 0, 0);
        SendMessage(hComboClickType, CB_ADDSTRING, 0, (LPARAM)L"Clic Izquierdo");
        SendMessage(hComboClickType, CB_ADDSTRING, 0, (LPARAM)L"Clic Derecho");
        SendMessage(hComboClickType, CB_ADDSTRING, 0, (LPARAM)L"Clic Central");
    } else {
        SetWindowTextW(hGroupSettings, L" Settings ");
        SetWindowTextW(hGroupStop, L" Stop Condition ");
        SetWindowTextW(hGroupHotkey, L" Global Hotkey ");
        SetWindowTextW(hLabelClickRate, L"Click Interval (ms):");
        SetWindowTextW(hLabelButtonType, L"Mouse Button:");
        SetWindowTextW(hRadioUntilStopped, L"Click until stopped manually");
        SetWindowTextW(hRadioUntilCount, L"Stop automatically after (clicks):");
        SetWindowTextW(hLabelKeybind, L"Press to Start/Stop:");
        SetWindowTextW(hEditStartButton, L"START");
        SetWindowTextW(hEditStopButton, L"STOP");
        SetWindowTextW(hButtonLang, L"ES");

        SendMessage(hComboClickType, CB_RESETCONTENT, 0, 0);
        SendMessage(hComboClickType, CB_ADDSTRING, 0, (LPARAM)L"Left Click");
        SendMessage(hComboClickType, CB_ADDSTRING, 0, (LPARAM)L"Right Click");
        SendMessage(hComboClickType, CB_ADDSTRING, 0, (LPARAM)L"Middle Click");
    }
    
    if (clickType != CB_ERR) {
        SendMessage(hComboClickType, CB_SETCURSEL, clickType, 0);
    } else {
        SendMessage(hComboClickType, CB_SETCURSEL, 0, 0);
    }
    
    InvalidateRect(m_hWnd, NULL, TRUE);
    UpdateWindow(m_hWnd);
}
