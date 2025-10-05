import sys
import io
import re
import pymupdf
from pymupdf4llm import to_markdown

def clean_tables(text):
    """Remove malformed tables with mostly empty cells"""
    # Remove malformed tables with mostly empty cells
    text = re.sub(r'\|[\|\s-]*\|[\n\r]*', '', text)
    # Remove standalone table separators
    text = re.sub(r'^[\|\-\s]+$', '', text, flags=re.MULTILINE)
    return text

def normalize_whitespace(text):
    """Normalize excessive whitespace from PDF conversion"""
    # Remove extra blank lines (keep max 2 consecutive)
    text = re.sub(r'\n\s*\n\s*\n+', '\n\n', text)
    # Remove trailing spaces
    text = re.sub(r' +$', '', text, flags=re.MULTILINE)
    # Normalize multiple spaces to single space
    text = re.sub(r' {2,}', ' ', text)
    return text

def clean_code_blocks(text):
    """Remove empty or meaningless code blocks"""
    # Remove empty code blocks
    text = re.sub(r'```\s*\n\s*```', '', text)
    # Remove code blocks with just email/simple content
    text = re.sub(r'```\s*\n\s*[\w@.-]+\s*\n\s*```', lambda m: m.group(0).strip('`\n '), text)
    return text

def consolidate_headers(text):
    """Reduce redundant formatting"""
    # Convert excessive header levels to simpler format
    text = re.sub(r'^#{4,} ', '### ', text, flags=re.MULTILINE)
    return text

def optimize_markdown(text):
    """Apply all optimization techniques to reduce token count"""
    text = clean_tables(text)
    text = clean_code_blocks(text)
    text = normalize_whitespace(text)
    text = consolidate_headers(text)
    
    # Remove excessive newlines at start/end
    text = text.strip()
    
    return text

def convert_to_markdown(file_path, optimize=True, **kwargs):
    """
    Convert a PDF file to markdown format using pymupdf4llm.
    
    Args:
        file_path: Path to the PDF file
        optimize: Whether to apply token optimization (default: True)
        **kwargs: Additional arguments passed to pymupdf4llm.to_markdown
    """
    try:
        # Try to open the PDF file
        doc = pymupdf.open(file_path)
       
        # Convert to markdown using pymupdf4llm
        markdown_text = to_markdown(doc, show_progress=False, **kwargs)
       
        # Close the document
        doc.close()
        
        # Apply optimization if requested
        if optimize:
            markdown_text = optimize_markdown(markdown_text)
       
        return True, markdown_text
   
    except FileNotFoundError:
        return False, f"Error: File '{file_path}' not found"
    except pymupdf.errors.FileDataError:
        return False, f"Error: '{file_path}' is not a valid PDF file"
    except Exception as e:
        return False, f"Error: {str(e)}"

if __name__ == "__main__":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
   
    if len(sys.argv) < 2:
        print("Error: No file path provided", file=sys.stderr)
        sys.exit(1)
       
    file_path = sys.argv[1]
    
    # Check for optimization flag
    optimize = True
    if len(sys.argv) > 2 and sys.argv[2] == "--no-optimize":
        optimize = False
    
    success, result = convert_to_markdown(file_path, optimize=optimize)
   
    if success:
        print(result)
        sys.exit(0)
    else:
        print(result, file=sys.stderr)
        sys.exit(1)