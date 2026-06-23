import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    # This regex finds all matches of strings and comments
    pattern = re.compile(r'(\".*?\"|\'.*?\'|`.*?`|//.*|/\*.*?\*/)', re.DOTALL)
    
    # But we want to find if any of them are UNCLOSED.
    # A regex match will only succeed if it finds a closing quote.
    # So if there's an unclosed quote, the regex won't match it, 
    # but the remaining text will contain the unclosed quote.
    
    # Let's try to find unclosed quotes by looking for quotes that don't match
    for quote in ['"', "'", "`"]:
        # This is a very naive way
        count = content.count(quote)
        if count % 2 != 0:
            print(f"Unbalanced {quote}")

solve()
