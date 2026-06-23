import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    pattern = r'(\".*?\"|\'.*?\'|`.*?`|//.*|/\*.*?\*/)'
    tokens = []
    last_end = 0
    for match in re.finditer(pattern, content, flags=re.DOTALL):
        tokens.append(('text', content[last_end:match.start()]))
        tokens.append(('comment_or_string', match.group()))
        last_end = match.end()
    tokens.append(('text', content[last_end:]))

    stack = []
    for token_type, text in tokens:
        if token_type == 'text':
            for match in re.finditer(r'\{|\}', text):
                char = match.group()
                if char == '{':
                    stack.append(match.start())
                elif char == '}':
                    if stack:
                        stack.pop()
                    else:
                        print(f"Unmatched closing brace at index {match.start()}")
                        return
    
    if stack:
        print(f"Unmatched opening brace at index {stack[0]}")
    else:
        print("All braces are balanced")

solve()
