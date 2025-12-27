import requests
from bs4 import BeautifulSoup
import time
import json

SWAPI_URL = "https://swapi.info/api/films"
WIKI_BASE_URL = "https://starwars.fandom.com/wiki"

FILM_SLUGS = {
    "The Phantom Menace": "Star_Wars:_Episode_I_The_Phantom_Menace",
    "Attack of the Clones": "Star_Wars:_Episode_II_Attack_of_the_Clones",
    "Revenge of the Sith": "Star_Wars:_Episode_III_Revenge_of_the_Sith",
    "A New Hope": "Star_Wars:_Episode_IV_A_New_Hope",
    "The Empire Strikes Back": "Star_Wars:_Episode_V_The_Empire_Strikes_Back",
    "Return of the Jedi": "Star_Wars:_Episode_VI_Return_of_the_Jedi"
}

def fetch_swapi_data():
    """Obtiene la data cruda desde la API swapi.info"""
    print(f"📡 Conectando a {SWAPI_URL}...")
    try:
        response = requests.get(SWAPI_URL)
        response.raise_for_status()
        data = response.json()
        print(f"✅ Datos obtenidos: {len(data)} películas encontradas.")
        return data
    except requests.exceptions.RequestException as e:
        print(f"❌ Error fetching data: {e}")
        return None

def fetch_wiki_context(title):
    """Busca la sinopsis en Wookieepedia basado en el título"""
    if title not in FILM_SLUGS:
        return "Wiki entry not mapped."
        
    slug = FILM_SLUGS[title]
    url = f"{WIKI_BASE_URL}/{slug}"
    print(f"   📖 Leyendo contexto de Wiki: {slug}...")
    
    try:
        # User-Agent es obligatorio para Fandom
        headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}
        response = requests.get(url, headers=headers)
        
        if response.status_code == 200:
            soup = BeautifulSoup(response.text, 'html.parser')
            content_div = soup.find('div', {'class': 'mw-parser-output'})
            
            if content_div:
                # Extraemos párrafos limpios, ignorando mensajes de navegación
                paragraphs = []
                for p in content_div.find_all('p', recursive=False):
                    text = p.get_text(strip=True)
                    # Filtro simple para evitar metadatos vacíos o cortos
                    if len(text) > 50 and not text.startswith("aside"):
                        paragraphs.append(text)
                        if len(paragraphs) >= 3: break 
                
                return "\n\n".join(paragraphs)
        return "No description found."
    except Exception as e:
        return f"Wiki Error: {e}"

def main():
    raw_films = fetch_swapi_data()
    
    if not raw_films:
        return

    processed_data = []

    sorted_films = sorted(raw_films, key=lambda x: x['episode_id'])

    for film in sorted_films:
        ep_id = film['episode_id']
        title = film['title']
        
        if 1 <= ep_id <= 6:
            print(f"⚙️ Procesando Episodio {ep_id}: {title}")
            
            wiki_summary = fetch_wiki_context(title)
            
            character_ids = [url.strip('/').split('/')[-1] for url in film['characters']]
            
            film_record = {
                "id": f"film_{ep_id}",
                "title": title,
                "episode_id": ep_id,
                "director": film['director'],
                "release_date": film['release_date'],
                "opening_crawl": film['opening_crawl'].replace("\r\n", " "),
                "wiki_plot": wiki_summary,
                "character_ids": character_ids,
                "source": "swapi_plus_wookieepedia"
            }
            
            processed_data.append(film_record)
            
            time.sleep(1)

    print(f"\n✨ Proceso finalizado. {len(processed_data)} películas listas.")
    
    return processed_data;

