import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    # This regex is a bit simplified but should work for most cases
    # It matches strings and comments
    pattern = r'(\".*?\"|\'.*?\'|`.*?`|//.*|/\*.*?\*/)'
    
    # We will iterate through the content and track our position
    stack = []
    pos = 0
    
    for match in re.finditer(pattern, content, flags=re.DOTALL):
        # Process text before the match
        text_before = content[pos:match.start()]
        for m in re.finditer(r'\{|\}', text_before):
            if m.group() == '{':
                stack.append(m.end()) # Store end position to help find it later
            else:
                if stack:
                    stack.pop()
                else:
                    print(f'Unmatched closing brace at index {m.start()}')
                    return
        pos = match.end()

    # Process remaining text
    text_after = content[pos:]
    for m in re.finditer(r'\{|\}', text_after):
        if m.group() == '{':
            stack.append(m.end())
        else:
            if stack:
                stack.pop()
            else:
                print(f'Unmatched closing brace at index {m.start()}')
                return

    if stack:
        print(f'Unmatched opening brace at index {stack[0]}')
    else:
        print('All braces are balanced')

solve()
