import fitz  # PyMuPDF
import sys
import os
from PIL import Image
import io

def extract_first_page(pdf_path, output_path, max_size=(800, 1200)):
    try:
        # Open the PDF
        doc = fitz.open(pdf_path)
        
        if doc.page_count < 1:
            raise ValueError("PDF is empty")

        # Get the first page
        page = doc[0]
        
        # Convert page to image with higher resolution
        pix = page.get_pixmap(matrix=fitz.Matrix(2, 2))
        
        # Convert to PIL Image for better processing
        img = Image.frombytes("RGB", [pix.width, pix.height], pix.samples)
        
        # Calculate new size maintaining aspect ratio
        ratio = min(max_size[0]/img.width, max_size[1]/img.height)
        new_size = (int(img.width * ratio), int(img.height * ratio))
        img = img.resize(new_size, Image.Resampling.LANCZOS)
        
        # Save the image
        img.save(output_path, "PNG", optimize=True)
        return True, "Successfully extracted cover"
        
    except Exception as e:
        return False, f"Error extracting cover: {str(e)}"
    finally:
        if 'doc' in locals():
            doc.close()

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python cover_extractor.py <pdf_path> <output_path>", file=sys.stderr)
        sys.exit(1)

    pdf_path = sys.argv[1]
    output_path = sys.argv[2]
    
    # Create output directory if it doesn't exist
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    success, msg = extract_first_page(pdf_path, output_path)
    if success:
        print(msg)
        sys.exit(0)
    else:
        print(msg, file=sys.stderr)
        sys.exit(1)