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

export async function cleanPDFContent(filePath: string): Promise<string> {
  try {
    console.log('Extracting and cleaning PDF content from:', filePath);
    // Ensure we're passing the file path, not the content
    const content = await window.electronAPI.parsePDF(filePath);
   
    // Simple cleaning of the extracted text
    const cleanedContent = content
      .replace(/(\r\n|\n|\r)/gm, " ") // Replace line breaks with spaces
      .replace(/\s+/g, " ") // Replace multiple spaces with single space
      .trim();
    console.log('PDF content extracted and cleaned successfully');
    return cleanedContent;
  } catch (error) {
    console.error('Error extracting and cleaning PDF content:', error);
    throw new Error(`Failed to extract and clean PDF content: ${error.message}`);
  }
}