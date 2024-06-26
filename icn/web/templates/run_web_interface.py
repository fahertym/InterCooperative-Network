import sys
import logging

print("Python version:", sys.version)
print("Python path:", sys.path)

logging.basicConfig(level=logging.DEBUG)

try:
    print("Attempting to import app...")
    from icn.web import app
    print("Import successful!")
    
    if __name__ == '__main__':
        print("Starting the web interface...")
        app.run(debug=True)
        print("Web interface has stopped.")
except ImportError as e:
    print(f"Import error: {e}")
    import traceback
    traceback.print_exc()
except Exception as e:
    print(f"An unexpected error occurred: {e}")
    import traceback
    traceback.print_exc()