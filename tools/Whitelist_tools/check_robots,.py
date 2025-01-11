import requests
from urllib.robotparser import RobotFileParser

def is_scraping_allowed(url, user_agent="*"):
    try:
        # Parse the domain's robots.txt
        domain = "/".join(url.split("/")[:3])  # Extract base domain
        robots_url = f"{domain}/robots.txt"
        
        response = requests.get(robots_url, timeout=5)
        if response.status_code == 200:
            # Parse the robots.txt file
            rp = RobotFileParser()
            rp.parse(response.text.splitlines())
            
            # Check if the user-agent is allowed to scrape the URL
            return rp.can_fetch(user_agent, url)
        else:
            print(f"No robots.txt found for {domain}. Assuming scraping is allowed.")
            return True  # Default to allowed if no robots.txt exists
    except Exception as e:
        print(f"Error checking robots.txt: {e}")
        return False

# Test the function
urls = [
    "https://www.wikipedia.org",
    "https://www.bbc.com/news",
    "https://www.si.edu",
    "https://www.data.gov",
    "https://www.linkedin.com",
    "https://www.instagram.com",
    "https://www.amazon.com",
    "https://www.google.com"
]

for url in urls:
    if is_scraping_allowed(url):
        print(f"Scraping is allowed for {url}")
    else:
        print(f"Scraping is NOT allowed for {url}")
