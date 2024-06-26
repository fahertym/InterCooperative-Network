from flask import Flask

print("Creating Flask app...")
app = Flask(__name__)
print("Flask app created.")

print("Importing routes...")
from . import routes
print("Routes imported.")