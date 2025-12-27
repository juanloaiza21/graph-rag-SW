import os
from typing import Dict, Any, List
from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pymongo import MongoClient
from dotenv import load_dotenv
from datetime import datetime
from services.characters_service import main as ingest_characters_data
from services.starships_service import main as ingest_starships_data

# Importamos tu servicio
from services.movies_service import main as ingest_movies_data

load_dotenv()

APP_TITLE = "Galactic Data Lake API"
VERSION = "0.1.0"

app = FastAPI(title=APP_TITLE, version=VERSION)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

MONGO_URI = f"mongodb://{os.getenv('MONGO_USER')}:{os.getenv(
    'MONGO_PASSWORD')}@localhost:{os.getenv('MONGO_PORT', '27017')}/"


def get_db_collection(collection_name: str):
    try:
        client = MongoClient(MONGO_URI, serverSelectionTimeoutMS=5000)
        client.admin.command('ping')
        db = client["raw_data_lake"]
        return db[collection_name]
    except Exception as e:
        print(f"❌ Error crítico conectando a Mongo: {e}")
        raise HTTPException(
            status_code=500, detail="Database Connection Error")

# ==========================================
# 🚦 ENDPOINTS
# ==========================================


@app.get("/")
def read_root():
    return {
        "system": APP_TITLE,
        "status": "online",
        "docs": "/docs"
    }


@app.get("/system/status", response_model=Dict[str, Any])
def check_system_status():
    """
    Revisa el estado de la colección de PELÍCULAS
    """
    try:
        collection = get_db_collection("movies_raw")
        doc_count = collection.count_documents({})

        return {
            "status": "operational",
            "database": "mongodb",
            "collection": "movies_raw",
            "has_data": doc_count > 0,
            "total_documents": doc_count,
            "checked_at": datetime.now()
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/system/seed/movies")
def seed_movies_data():
    """
    Seeder para Películas: Borra lo existente y carga data fresca
    """
    try:
        collection = get_db_collection("movies_raw")

        print("⏳ Extrayendo datos de SWAPI + Wiki...")
        data = ingest_movies_data()

        if not data:
            return {"status": False, "message": "No data returned from service"}

        delete_result = collection.delete_many({})
        insert_result = collection.insert_many(data)

        return {
            "status": True,
            "action": "seed_movies",
            "deleted_previous": delete_result.deleted_count,
            "inserted_count": len(insert_result.inserted_ids),
            "example_id": str(insert_result.inserted_ids[0])
        }

    except Exception as e:
        print(f"❌ Error en Seed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/system/seed/characters")
def seed_characters_data():
    """
    Seeder para Personajes
    """
    try:
        collection = get_db_collection("characters_raw")

        print("⏳ Iniciando protocolo de ingesta de droides y seres orgánicos...")
        data = ingest_characters_data()

        if not data:
            return {"status": False, "message": "No data returned from characters service"}

        delete_result = collection.delete_many({})
        insert_result = collection.insert_many(data)

        return {
            "status": True,
            "action": "seed_characters",
            "deleted_previous": delete_result.deleted_count,
            "inserted_count": len(insert_result.inserted_ids),
            "sample_id": str(insert_result.inserted_ids[0])
        }

    except Exception as e:
        print(f"❌ Error en Seed Characters: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/system/seed/starships")
def seed_starships_data():
    """
    Seeder para Naves (Starships):
    Filtra naves de Ep 1-6, busca en Wiki y guarda en 'starships_raw'.
    """
    try:
        collection = get_db_collection("starships_raw")

        print("⏳ Inicializando sistemas de navegación...")
        data = ingest_starships_data()

        if not data:
            return {"status": False, "message": "No starships data returned"}

        delete_result = collection.delete_many({})

        insert_result = collection.insert_many(data)

        return {
            "status": True,
            "action": "seed_starships",
            "deleted_previous": delete_result.deleted_count,
            "inserted_count": len(insert_result.inserted_ids),
            "sample_id": str(insert_result.inserted_ids[0])
        }

    except Exception as e:
        print(f"❌ Error en Seed Starships: {e}")
        raise HTTPException(status_code=500, detail=str(e))
