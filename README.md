# StudyCraft

<div align="center">

<img width="170" height="170" alt="icon" src="https://github.com/user-attachments/assets/ac0b58ef-2855-44cc-a6f2-7dde8b7985b5" />


StudyCraft is a multi-platform desktop application that lets you create study materials from a wide range of sources, edit them as you wish, generate flashcard decks and tests either manually or automatically via LLMs, track your study sessions and customize everything. Built with [Rust](https://rust-lang.org/) and [Svelte](https://svelte.dev/) via the [Tauri](https://v2.tauri.app) framework.

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://v2.tauri.app/)
[![Svelte](https://img.shields.io/badge/Svelte-5-orange.svg)](https://svelte.dev/)

</div>

## Table of Contents
- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
  - [Using an installer](#using-an-installer)
  - [Building from source](#building-from-source)
- [Transfering/Deleting your Data](#transferingdeleting-your-data)
- [Project Structure](#project-structure)
- [License](#license)
- [Contributing](#contributing)
- [Supporting StudyCraft](#supporting-studycraft)

## Features

- Study Materials
    - Import PDFs, Markdown or TXT files, as well as Websites or Github repos.
    - Automatic conversion and markdown editing.
    - All materials are automatically indexed in a vector database for semantic search.

- Review Materials
    - Create flashcard decks and tests manually or auto-generate from study materials
    - Use of an Anki-styled space repetition

- AI Integrations
    - Choose from OpenAI, Google, Anthropic, Mistral, or Ollama. Use multiple LLM providers simultaneously for faster generation.
    - Generate Review Materials automatically from your Study Materials
    - Ask questions about any of your study materials and get in-depth explanations on any topic

- Progress Tracking
    - Visualize your study sessions with charts.
    - Track your learning journey over time. Monitor your progress and improvement

- Customization
    - Deep configuration for spaced repetition and AI settings.

### Screenshots

<div align="center">
<img width="686" height="479" alt="img1" src="https://github.com/user-attachments/assets/49cb0a73-2d64-49ab-a0a6-90a67c28862e" />
<img width="696" height="452" alt="img2" src="https://github.com/user-attachments/assets/901a7a4d-ffff-4cee-a454-4c74afb1a1e9" />
<img width="697" height="482" alt="img3" src="https://github.com/user-attachments/assets/c1cfaf41-f38c-4e69-90f4-130618f465ad" />
<img width="694" height="488" alt="img4" src="https://github.com/user-attachments/assets/f291489d-0180-4428-8f81-bad421cee855" />

</div>

## Installation

### Using an installer

Click here to download the latest release. StudyCraft is available on Windows, MacOS and Linux. 

### Building from source

Requirements:
- **Node.js** (v18 or higher)
- **Rust** (latest stable version)
- **npm** or **pnpm**
- **Python 3.10**

1. **Clone the repository**
   ```bash
   git clone https://github.com/rodmarkun/StudyCraft
   cd StudyCraft
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Set up Python environment**
    ```python
    pip install pyinstaller pymupdf4llm pillow
    ```

4. **Embed Python binaries**

You need to build the Python executables that will be bundled with the app.

*On Linux*:
   ```bash
   cd src-tauri
   mkdir -p dist
   
   # Build conversor
   pyinstaller conversor.spec --distpath pyinstaller_dist
   mv pyinstaller_dist/conversor dist/conversor-x86_64-unknown-linux-gnu
   chmod +x dist/conversor-x86_64-unknown-linux-gnu
   
   # Build cover-extractor
   pyinstaller cover-extractor.spec --distpath pyinstaller_dist
   mv pyinstaller_dist/cover-extractor dist/cover-extractor-x86_64-unknown-linux-gnu
   chmod +x dist/cover-extractor-x86_64-unknown-linux-gnu
   
   cd ..
   ```

*On MacOS (Apple Silicon)*
   ```bash
   cd src-tauri
   mkdir -p dist
   
   # Build conversor
   pyinstaller conversor.spec --distpath pyinstaller_dist
   mv pyinstaller_dist/conversor dist/conversor-aarch64-apple-darwin
   chmod +x dist/conversor-aarch64-apple-darwin
   
   # Build cover-extractor
   pyinstaller cover-extractor.spec --distpath pyinstaller_dist
   mv pyinstaller_dist/cover-extractor dist/cover-extractor-aarch64-apple-darwin
   chmod +x dist/cover-extractor-aarch64-apple-darwin
   
   cd ..
   ```

*On Windows (CMD)*
   ```cmd
   cd src-tauri
   mkdir dist
   
   rem Build conversor
   pyinstaller conversor.spec --distpath pyinstaller_dist
   move /Y pyinstaller_dist\conversor.exe dist\conversor-x86_64-pc-windows-msvc.exe
   
   rem Build cover-extractor
   pyinstaller cover-extractor.spec --distpath pyinstaller_dist
   move /Y pyinstaller_dist\cover-extractor.exe dist\cover-extractor-x86_64-pc-windows-msvc.exe
   
   cd ..
   ```

5. **Run in development mode**
   ```bash
   npm run tauri dev
   ```

6. **Build for production**
   ```bash
   npm run tauri build
   ```

## Transfering/Deleting your Data

All of your data is stored locally. The only thing that travels through the internet are chunks of the documents you choose to generate review materials from via LLMs. You can access your data in the following directory, depending on your OS:

- Windows: \C:\Users\{USER}\AppData\StudyCraft
- macOS: ~/Library/Application Support/StudyCraft
- Linux: ~/.local/share/StudyCraft

## Project Structure

```
studycraft-v2/
├── src/                         # Frontend (Svelte)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Layout/          # App layout components
│   │   │   ├── ReviewMaterial/  # Flashcards & tests
│   │   │   ├── Settings/        # Configuration UI
│   │   │   ├── Shared/          # Reusable components
│   │   │   ├── Stats/           # Statistics & charts
│   │   │   └── StudyMaterials/  # Material management
│   │   ├── logic/               # Business logic
│   │   ├── stores/              # Svelte stores (state)
│   │   └── styles/              # Global styles
│   └── App.svelte               # Root component
│
├── src-tauri/                   # Backend (Rust)
│   ├── src/
│   │   ├── commands/            # Tauri commands (frontend API)
│   │   │   ├── api.rs           # API-related commands
│   │   │   ├── llm.rs           # LLM integration commands
│   │   │   ├── materials/       # Material management commands
│   │   │   ├── settings.rs      # Settings commands
│   │   │   ├── structure/       # Structure/organization commands
│   │   │   ├── tags.rs          # Tagging system commands
│   │   │   └── vector.rs        # Vector/embedding commands
│   │   │
│   │   ├── materials/           # Material domain models
│   │   │
│   │   ├── services/            # Business logic services
│   │   │   ├── db_service/      # Database operations
│   │   │   ├── file_service/    # File conversion operations
│   │   │   ├── llm_service/     # LLM provider integration
│   │   │   ├── url_service/     # URL/web content handling
│   │   │   └── vector_service/  # Vector embeddings & search
│   │   │
│   │   ├── config/              # Configuration management
│   │   │   ├── api_config.rs    # API configuration
│   │   │   ├── app_settings.rs  # App settings
│   │   │   ├── keystore.rs      # Secure key storage
│   │   │   └── paths.rs         # Path management
│   │   │
│   │   ├── constants.rs         # App constants
│   │   ├── errors.rs            # Error types & handling
│   │   ├── state.rs             # App state management
│   │   ├── utils.rs             # Utility functions
│   │   └── main.rs              # Entry point
│   │
│   └── tauri.conf.json          # Tauri configuration
│
└── public/                      # Static assets
```

## License

This project is licensed under the **Apache License 2.0** - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are always welcome! If you're interested in contributing to StudyCraft, please fork the repository and create a new branch for your changes. When you're done with your changes, submit a pull request to merge your changes into the main branch.

## Supporting StudyCraft

If you want to support StudyCraft, you can:
- **Star** :star: the project in Github!
- **Donate** :coin: to my [Ko-fi](https://ko-fi.com/rodmarkun) page!
- **Share** :heart: the project with your friends!

<div align="center">

**Made with ❤️ by @rodmarkun**

</div>
