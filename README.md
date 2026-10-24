<div align="center">
  <img src="Autoclicker.ico" alt="Logo" width="100" height="100">
  
  # NanoClick 🖱️⚡

  **Un autoclicker ultra-rápido, ligero y nativo para Windows.**

  [![C++](https://img.shields.io/badge/C%2B%2B-17%2B-blue.svg)](https://isocpp.org/)
  [![Platform](https://img.shields.io/badge/Platform-Windows%20Win32-lightgrey.svg)]()
  [![Architecture](https://img.shields.io/badge/Architecture-Clean%20Code-success.svg)]()

</div>

---

## 📖 Acerca del Proyecto

> **Reescritura Rust en planificación:** la implementación actual sigue siendo C++/Win32. Consulta el [plan de migración](tasks/plan.md), la [arquitectura Rust](docs/architecture/RUST.md), las [tareas](tasks/todo.md) y el [estado verificado](docs/STATUS.md). El soporte Linux y los objetivos de rendimiento de la nueva versión todavía están pendientes de implementación y pruebas.

**NanoClick** es una herramienta desarrollada nativamente con la API Win32 de Windows, diseñada para ser increíblemente rápida y no consumir recursos. A diferencia de otros autoclickers hechos en Python o C#, este programa se ejecuta directamente a bajo nivel, garantizando cero lag y máxima precisión gracias a la implementación de *High Resolution Timers*.

## ✨ Características

- 🚀 **Ultra-rápido**: Precisión al milisegundo mediante `timeBeginPeriod(1)`.
- 🪶 **Ligero**: Sin dependencias externas, no requiere instalar .NET, Java, ni nada.
- 🌎 **Bilingüe**: Interfaz completamente en Español e Inglés en tiempo real.
- 🎯 **Modos de Parada**: Haz clic infinitamente o detente exactamente luego de N clics.
- ⌨️ **Atajos Globales**: Configura tu propia tecla (por defecto `F6`) para iniciar o detener en cualquier momento.

## 🛠️ Instalación y Compilación

Este proyecto está construido para **Visual Studio** (recomendado VS 2022).

1. Clona el repositorio:
   ```bash
   git clone https://github.com/bensalcos/Autoclicker.git
   ```
2. Abre la solución `Autoclicker.sln` en Visual Studio.
3. Asegúrate de tener instalado el *Workload* **Desarrollo para el escritorio con C++**.
4. Selecciona configuración `Release` y compila la solución (`Ctrl + Shift + B`).
5. El ejecutable compilado estará listo para ser utilizado en la carpeta `x64/Release/`.

---

