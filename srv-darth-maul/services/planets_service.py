import requests
from bs4 import BeautifulSoup
import time
import json

# --- CONFIGURACIÓN ---
SWAPI_FILMS_URL = "https://swapi.info/api/films"
SWAPI_PLANETS_URL = "https://swapi.info/api/planets"
WIKI_BASE_URL = "https://starwars.fandom.com/wiki"

PLANET_NAME_EXCEPTIONS = {
    "Yavin IV": "Yavin_4",
    "Dantooine": "Dantooine",
    "Bestine IV": "Bestine_IV"
}


def get_target_planet_ids():
    """
    Obtiene los IDs de planetas que aparecen en Episodios 1-6.
    """
    print("🌍 Escaneando cartas de navegación (Ep 1-6)...")
    try:
        response = requests.get(SWAPI_FILMS_URL)
        films = response.json()

        target_ids = set()

        for film in films:
            if 1 <= film['episode_id'] <= 6:
                for planet_url in film['planets']:
                    planet_id = planet_url.strip('/').split('/')[-1]
                    target_ids.add(planet_id)

        return target_ids
    except Exception as e:
        print(f"❌ Error obteniendo IDs de películas: {e}")
        return set()


def fetch_wiki_data(name):
    """Scraping de Wookieepedia para Planetas"""

    # 1. Resolver Slug
    if name in PLANET_NAME_EXCEPTIONS:
        slug = PLANET_NAME_EXCEPTIONS[name]
    else:
        slug = name.replace(" ", "_")

    url = f"{WIKI_BASE_URL}/{slug}"

    try:
        headers = {'User-Agent': 'Mozilla/5.0'}
        response = requests.get(url, headers=headers)

        if response.status_code == 200:
            soup = BeautifulSoup(response.text, 'html.parser')
            parser = soup.find('div', {'class': 'mw-parser-output'})

            if parser:
                paragraphs = []
                for p in parser.find_all('p', recursive=False):
                    text = p.get_text(strip=True)
                    # Filtramos intros vacías o de navegación
                    if len(text) > 40 and not text.lower().startswith("aside"):
                        paragraphs.append(text)
                        if len(paragraphs) >= 5:
                            break

                return "\n\n".join(paragraphs)
        return "Geological data available, but history description missing."
    except Exception as e:
        return f"Wiki Error: {str(e)}"


def main():
    # 1. Obtener whitelist de IDs
    valid_ids = get_target_planet_ids()
    if not valid_ids:
        return []

    # 2. Obtener TODOS los planetas
    print("📡 Descargando base de datos galáctica de planetas...")
    try:
        response = requests.get(SWAPI_PLANETS_URL)
        all_planets = response.json()
    except Exception as e:
        print(f"❌ Error conectando a SWAPI Planets: {e}")
        return []

    processed_planets = []

    print("🪐 Analizando ecosistemas...")

    for planet in all_planets:
        planet_id = planet['url'].strip('/').split('/')[-1]

        if planet_id in valid_ids:
            name = planet['name']

            # Enriquecimiento
            wiki_text = fetch_wiki_data(name)

            # Limpieza de relaciones
            film_ids = [f.strip('/').split('/')[-1]
                        for f in planet.get('films', [])]
            resident_ids = [r.strip('/').split('/')[-1]
                            for r in planet.get('residents', [])]

            planet_obj = {
                "id": f"planet_{planet_id}",
                "original_swapi_id": planet_id,
                "name": name,
                "rotation_period": planet.get('rotation_period'),
                "orbital_period": planet.get('orbital_period'),
                "diameter": planet.get('diameter'),
                "climate": planet.get('climate'),
                "gravity": planet.get('gravity'),
                "terrain": planet.get('terrain'),
                "surface_water": planet.get('surface_water'),
                "population": planet.get('population'),
                "wiki_description": wiki_text,
                "film_ids": film_ids,
                "resident_ids": resident_ids,  # Relación con Personajes
                "source": "swapi_plus_wookieepedia"
            }

            processed_planets.append(planet_obj)
            time.sleep(0.5)

    print(f"\n✅ Finalizado. {len(processed_planets)} planetas cartografiados.")
    return processed_planets


if __name__ == "__main__":
    res = main()
    if res:
        print(json.dumps(res[0], indent=2))
