#!/bin/bash

echo "=========================================="
echo "🛠️  Iniciando configuración del Entorno..."
echo "=========================================="

# 1. Verificar si existe la carpeta venv
if [ ! -d "venv" ]; then
    echo "📦 Creando entorno virtual (venv)..."
    python3 -m venv venv
else
    echo "✅ El entorno virtual ya existe."
fi

# 2. Activar el entorno (Truco para scripts: source no siempre persiste al salir, 
# pero aquí lo usamos para ejecutar los comandos siguientes dentro del entorno)
source venv/bin/activate

# 3. Instalación
echo "⬇️  Instalando dependencias desde requirements.txt..."
pip install --upgrade pip
pip install -r requirements.txt

echo "=========================================="
echo "🚀 ¡Instalación completada!"
echo "Para activar el entorno manualmente usa:"
echo "source venv/bin/activate"
echo "=========================================="
