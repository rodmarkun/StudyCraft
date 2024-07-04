// src/electron.d.ts
interface ElectronAPI {
    saveFile: (content: string, fileName: string, collectionName: string) => Promise<string>;
    readFile: (filePath: string) => Promise<string>;
    deleteFile: (filePath: string) => Promise<void>;
  }
  
  declare global {
    interface Window {
      electronAPI: ElectronAPI;
    }
  }