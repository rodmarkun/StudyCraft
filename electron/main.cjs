// Modules to control application life and create native browser window
const { log } = require('console');
const { app, BrowserWindow, ipcMain, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const pdf = require('pdf-parse');
const axios = require('axios')

if (require('electron-squirrel-startup')) app.quit();

const isDevEnvironment = process.env.DEV_ENV === 'true';

// enable live reload for electron in dev mode
if (isDevEnvironment) {
    require('electron-reload')(__dirname, {
        electron: path.join(__dirname, '..', 'node_modules', '.bin', 'electron'),
        hardResetMethod: 'exit'
    });
}

let mainWindow;

const createWindow = () => {
    
    // Create the browser window.
    mainWindow = new BrowserWindow({
        width: 1300,
        height: 600,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            nodeIntegration: false
        },
        icon: path.join(__dirname, '..', 'src', 'assets', 'StudyCraftLogo.ico')
    })

    mainWindow.setMenu(null)

    // define how electron will load the app
    if (isDevEnvironment) {

        // if your vite app is running on a different port, change it here
        mainWindow.loadURL('http://localhost:5173/');

        // Open the DevTools.
        mainWindow.webContents.on("did-frame-finish-load", () => {
            mainWindow.webContents.openDevTools();
        });

        log('Electron running in dev mode: 🧪')

    } else {
        
        // when not in dev mode, load the build file instead
        mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));

        log('Electron running in prod mode: 🚀')
    }
}

ipcMain.handle('saveFile', (event, content, fileName, collectionName) => {
  const userDataPath = app.getPath('userData');
  const collectionPath = path.join(userDataPath, 'collections', collectionName);
  
  if (!fs.existsSync(collectionPath)) {
    fs.mkdirSync(collectionPath, { recursive: true });
  }
  
  const filePath = path.join(collectionPath, fileName);
  
  if (content instanceof ArrayBuffer) {
    // Handle binary data (e.g., PDF)
    fs.writeFileSync(filePath, Buffer.from(content));
  } else {
    // Handle text data (e.g., Markdown)
    fs.writeFileSync(filePath, content, 'utf-8');
  }
  
  return filePath;
});
  
  ipcMain.handle('readFile', (event, filePath) => {
    try {
      const userDataPath = app.getPath('userData');
      const collectionsPath = path.join(userDataPath, 'collections');
      
      let trueFilePath;
      if (filePath.startsWith(userDataPath)) {
        // If the filePath already includes the userData path, use it as is
        trueFilePath = filePath;
      } else {
        // If not, append it to the collections path
        trueFilePath = path.join(collectionsPath, filePath);
      }
  
      // Ensure the file is within the userData directory (security measure)
      if (!trueFilePath.startsWith(userDataPath)) {
        throw new Error('Access denied: Attempting to read file outside of userData directory');
      }
  
      const content = fs.readFileSync(trueFilePath, 'utf-8');
      return content;
    } catch (error) {
      console.error('Error reading file:', error);
      throw error;
    }
  });
  
  ipcMain.handle('deleteFile', (event, filePath) => {
    try {
      fs.unlinkSync(filePath);
      return true;
    } catch (error) {
      console.error('Error deleting file:', error);
      throw error;
    }
  });

  ipcMain.handle('openFile', (event, filePath) => {
    shell.openPath(filePath);
  });

  ipcMain.handle('parsePDF', async (event, filePath) => {
    try {
      console.log('Parsing PDF file:', filePath); // Add this log
      const dataBuffer = fs.readFileSync(filePath);
      const data = await pdf(dataBuffer);
      return data.text;
    } catch (error) {
      console.error('Error parsing PDF:', error);
      throw error;
    }
  });

  ipcMain.handle('saveStudySession', (event, session) => {
    const userDataPath = app.getPath('userData');
    const sessionsPath = path.join(userDataPath, 'studySessions.json');
    
    let sessions = [];
    if (fs.existsSync(sessionsPath)) {
      const content = fs.readFileSync(sessionsPath, 'utf-8');
      sessions = JSON.parse(content);
    }
    
    sessions.push(session);
    fs.writeFileSync(sessionsPath, JSON.stringify(sessions));
    
    return true;
  });
  
  ipcMain.handle('loadStudySessions', () => {
    const userDataPath = app.getPath('userData');
    const sessionsPath = path.join(userDataPath, 'studySessions.json');
    
    if (fs.existsSync(sessionsPath)) {
      const content = fs.readFileSync(sessionsPath, 'utf-8');
      return JSON.parse(content);
    }
    
    return [];
  });

  ipcMain.handle('deleteAllData', async () => {
    const userDataPath = app.getPath('userData');
    const collectionsPath = path.join(userDataPath, 'collections');
    const studySessionPath = path.join(userDataPath, 'studySessions.json');
  
    try {
      // Delete collections directory
      if (fs.existsSync(collectionsPath)) {
        fs.rmdirSync(collectionsPath, { recursive: true });
      }
  
      // Delete studySessions.json
      if (fs.existsSync(studySessionPath)) {
        fs.unlinkSync(studySessionPath);
      }
  
      return true;
    } catch (error) {
      console.error('Error deleting all data:', error);
      return false;
    }
  });

  ipcMain.handle('fetchWebContent', async (event, url) => {
    try {
      const response = await axios.get(url);
      return response.data;
    } catch (error) {
      console.error('Error fetching web content:', error);
      throw error;
    }
  });

  ipcMain.handle('renameCollectionFolder', async (event, oldName, newName) => {
    const userDataPath = app.getPath('userData');
    const oldPath = path.join(userDataPath, 'collections', oldName);
    const newPath = path.join(userDataPath, 'collections', newName);
    
    return new Promise((resolve, reject) => {
      fs.rename(oldPath, newPath, (err) => {
        if (err) {
          console.error('Error renaming collection folder:', err);
          reject(err);
        } else {
          // Update file paths within the renamed folder
          fs.readdir(newPath, (err, files) => {
            if (err) {
              console.error('Error reading directory:', err);
              reject(err);
            } else {
              files.forEach(file => {
                const oldFilePath = path.join(oldPath, file);
                const newFilePath = path.join(newPath, file);
                fs.rename(oldFilePath, newFilePath, err => {
                  if (err) console.error('Error renaming file:', err);
                });
              });
              resolve(true);
            }
          });
        }
      });
    });
  });
  
  ipcMain.handle('createCollectionFolder', async (event, name) => {
    const userDataPath = app.getPath('userData');
    const collectionPath = path.join(userDataPath, 'collections', name);
    
    return new Promise((resolve, reject) => {
      fs.mkdir(collectionPath, { recursive: true }, (err) => {
        if (err) {
          console.error('Error creating collection folder:', err);
          reject(err);
        } else {
          resolve(true);
        }
      });
    });
  });
  
  ipcMain.on('exitApp', () => {
    app.exit(0);
  });
  
  

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow);

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});

app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
// app.on('window-all-closed', () => {
//     if (process.platform !== 'darwin') app.quit()
// })

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.