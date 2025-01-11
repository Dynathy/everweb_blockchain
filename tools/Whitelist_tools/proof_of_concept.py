import hashlib
import requests
from bs4 import BeautifulSoup
from difflib import SequenceMatcher
import time
from selenium import webdriver
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.chrome.options import Options


def scrape_page_with_selenium(url, driver_path="path/to/chromedriver"):
    """
    Use Selenium to scrape a webpage in a headless browser.
    """
    options = Options()
    options.add_argument("--headless")  # Run in headless mode
    options.add_argument("--disable-gpu")  # Disable GPU rendering
    options.add_argument("--no-sandbox")  # Required for some Linux environments

    driver = webdriver.Chrome(service=Service(driver_path), options=options)

    try:
        driver.get(url)
        time.sleep(2)  # Allow time for dynamic content to load
        html = driver.page_source
        return html
    finally:
        driver.quit()


def scrape_page(url, retries=3, use_selenium=False, driver_path="path/to/chromedriver"):
    """
    Scrape a webpage using requests or Selenium.
    """
    if use_selenium:
        return scrape_page_with_selenium(url, driver_path)

    for attempt in range(retries):
        try:
            response = requests.get(url, timeout=10)
            response.raise_for_status()
            return response.text
        except requests.exceptions.RequestException as e:
            print(f"Error scraping {url}: {e}. Retrying... ({attempt + 1}/{retries})")
            time.sleep(2)
    print(f"Failed to scrape {url} after {retries} attempts.")
    return None


def save_html(content, filename):
    """
    Save raw HTML content to a file for inspection.
    """
    try:
        with open(filename, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Saved raw HTML to {filename}")
    except Exception as e:
        print(f"Error saving HTML to {filename}: {e}")


def canonicalize_html(html):
    """
    Clean and standardize HTML to reduce noise from dynamic content.
    """
    soup = BeautifulSoup(html, "html.parser")

    # Remove <script>, <style>, and other non-content tags
    for tag in soup(["script", "style"]):
        tag.decompose()

    # Remove dynamic elements (e.g., ads or timestamps)
    for tag in soup.find_all(attrs={"class": lambda x: x and "ad" in x.lower()}):
        tag.decompose()

    # Normalize IDs and classes
    for tag in soup.find_all(True):
        tag.attrs.pop("id", None)
        tag.attrs.pop("class", None)

    # Focus on main content if available
    main_content = soup.find("main")
    return main_content.get_text(separator=" ", strip=True) if main_content else soup.get_text(separator=" ", strip=True)


def hash_content(content):
    """
    Generate a SHA-256 hash of the given content.
    """
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


def is_similar(content1, content2, threshold=0.9):
    """
    Compare two pieces of content for similarity.
    """
    ratio = SequenceMatcher(None, content1, content2).ratio()
    return ratio >= threshold


def check_websites(websites, use_selenium=False, driver_path="path/to/chromedriver"):
    """
    Scrape each website twice, hash the content, and compare hashes.
    """
    results = {}

    for url in websites:
        print(f"Scraping the page: {url}")

        # First scrape
        print("Performing the first scrape...")
        raw_content1 = scrape_page(url, use_selenium=use_selenium, driver_path=driver_path)
        if not raw_content1:
            print(f"First scrape failed for {url}. Skipping...")
            results[url] = "First scrape failed"
            continue

        # Save raw HTML for inspection
        save_html(raw_content1, f"{url.split('//')[1].replace('/', '_')}_first.html")

        # Canonicalize and hash
        content1 = canonicalize_html(raw_content1)
        hash1 = hash_content(content1)
        print(f"First scrape hash for {url}: {hash1}")

        # Second scrape
        print("Performing the second scrape...")
        raw_content2 = scrape_page(url, use_selenium=use_selenium, driver_path=driver_path)
        if not raw_content2:
            print(f"Second scrape failed for {url}. Skipping...")
            results[url] = "Second scrape failed"
            continue

        # Save raw HTML for inspection
        save_html(raw_content2, f"{url.split('//')[1].replace('/', '_')}_second.html")

        # Canonicalize and hash
        content2 = canonicalize_html(raw_content2)
        hash2 = hash_content(content2)
        print(f"Second scrape hash for {url}: {hash2}")

        # Compare hashes
        if hash1 == hash2:
            print(f"The two scrapes match for {url}. The page content is consistent.")
            results[url] = "Match"
        elif is_similar(content1, content2):
            print(f"The two scrapes do NOT match for {url}, but the content is similar.")
            results[url] = "Similar"
            save_html(content1, f"{url.split('//')[1].replace('/', '_')}_canonical_first.txt")
            save_html(content2, f"{url.split('//')[1].replace('/', '_')}_canonical_second.txt")
        else:
            print(f"The two scrapes do NOT match for {url}. Significant content differences detected.")
            results[url] = "Mismatch"
            save_html(content1, f"{url.split('//')[1].replace('/', '_')}_canonical_first.txt")
            save_html(content2, f"{url.split('//')[1].replace('/', '_')}_canonical_second.txt")

        print("-" * 50)

    return results


def print_summary(results):
    """
    Print a summary of the results.
    """
    print("\nSummary of Results:")
    for url, status in results.items():
        print(f"{url}: {status}")


# List of websites to check
websites = [
    "https://www.wikipedia.org",
    "https://www.bbc.com/news",
    "https://www.si.edu",
    "https://www.data.gov"
]

# Path to your ChromeDriver
driver_path = "Whitelist_tools/chromedriver/chromedriver.exe"

# Check the websites
results = check_websites(websites, use_selenium=True, driver_path=driver_path)

# Print the results
print_summary(results)
