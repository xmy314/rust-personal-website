set -euo pipefail
IFS=$'\n\t'

rustup target add wasm32-unknown-unknown

echo "Building frontend with trunk."
cd frontend
trunk build
cd ..

echo "Copying frontend output to server/dist/."
rm -rf server/dist
mkdir -p server/dist
mv dist server

echo "Escape server/dist/ from gitignore in root."
touch server/.gitignore
echo "!dist/" >> server/.gitignore

echo "Build complete."
