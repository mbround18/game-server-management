# Use an official Python runtime as a parent image
FROM python:3.13-slim

# Prevent Python from writing pyc files and enable unbuffered output
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1

# Set the working directory
WORKDIR /app

# Copy requirements.txt and install dependencies
COPY requirements.txt /app/
RUN pip install --upgrade pip && pip install -r requirements.txt && apt update && apt install -y git && git config --global --add safe.directory /github/workspace

# Copy the rest of the application code
COPY . /app/

# Set the entrypoint to run your version manager script.
# Adjust the script filename if necessary.
ENTRYPOINT ["python", "/app/main.py"]
