from flask import Flask
import os

print("Creating Flask app...")
app = Flask(__name__)
app.secret_key = os.environ.get('SECRET_KEY') or 'you-will-never-guess'
print("Flask app created and secret key set.")

print("Importing routes...")
from . import routes
print("Routes imported.")