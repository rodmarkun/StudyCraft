// src/lib/utils/fileUtils.ts

export async function saveFile(content: string, fileName: string, collectionName: string): Promise<string> {
    console.log('Saving file:', fileName, 'to collection:', collectionName);
    const filePath = await window.electronAPI.saveFile(content, fileName, collectionName);
    console.log('File saved at:', filePath);
    return filePath;
  }
  
  export async function readFile(filePath: string): Promise<string> {
    console.log('Reading file from:', filePath);
    const content = await window.electronAPI.readFile(filePath);
    console.log('File content read, length:', content?.length);
    return content;
  }
  
  export async function deleteFile(filePath: string): Promise<boolean> {
    console.log('Deleting file:', filePath);
    const result = await window.electronAPI.deleteFile(filePath);
    console.log('File deleted:', result);
    return result;
  }