const { contextBridge, ipcRenderer } = require('electron')

const api = {
    node: () => process.versions.node,
    chrome: () => process.versions.chrome,
    electron: () => process.versions.electron
}

contextBridge.exposeInMainWorld('api', api)

contextBridge.exposeInMainWorld('electronAPI', {
  saveFile: (content, fileName, collectionName) =>
    ipcRenderer.invoke('saveFile', content, fileName, collectionName),
  readFile: (filePath) =>
    ipcRenderer.invoke('readFile', filePath),
  deleteFile: (filePath) =>
    ipcRenderer.invoke('deleteFile', filePath),
  saveStudySession: (session) =>
    ipcRenderer.invoke('saveStudySession', session),
  loadStudySessions: () =>
    ipcRenderer.invoke('loadStudySessions'),
  openFile: (filePath) => ipcRenderer.invoke('openFile', filePath),
  deleteAllData: () => ipcRenderer.invoke('deleteAllData'),
  exitApp: () => ipcRenderer.send('exitApp'),
  parsePDF: (filePath) => ipcRenderer.invoke('parsePDF', filePath),
  fetchWebContent: (url) => ipcRenderer.invoke('fetchWebContent', url),
  renameCollectionFolder: (oldName, newName) => ipcRenderer.invoke('renameCollectionFolder', oldName, newName),
  createCollectionFolder: (name) => ipcRenderer.invoke('createCollectionFolder', name),
});