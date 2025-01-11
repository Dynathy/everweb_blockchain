import requests

# List of candidate websites
websites = [
    "https://www.wikipedia.org",
    "https://www.bbc.com/news",
    "https://www.si.edu",
    "https://www.data.gov"
]

whitelist = []

for url in websites:
    try:
        # Check if the website is accessible
        response = requests.get(url, timeout=5)
        if response.status_code == 200:
            whitelist.append({"name": url.split("//")[1], "url": url, "status": "Accessible"})
    except Exception as e:
        print(f"Error accessing {url}: {e}")

# Generate JSON whitelist
import json
with open("whitelist.json", "w") as f:
    json.dump({"whitelist": whitelist}, f, indent=4)

print("Whitelist generated: whitelist.json")
