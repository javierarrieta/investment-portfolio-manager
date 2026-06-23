import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    pattern = r'(\/\*.*?\*\/|\/\/.*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|`(?:\\.|[^`\\])*`)'
    
    tokens = []
    last_end = 0
    for match in re.finditer(pattern, content, flags=re.DOTALL):
        tokens.append(('text', content[last_end:match.start()]))
        tokens.append(('other', match.group()))
        last_end = match.end()
    tokens.append(('text', content[last_end:]))

    stack = []
    for token_type, text in tokens:
        if token_type == 'text':
            for match in re.finditer(r'\{|\}', text):
                if match.group() == '{':
                    stack.append(match.start())
                else:
                    if stack:
                        stack.pop()
                    else:
                        print(f"Unmatched }}")
                        return
    
    if stack:
        print(f"Unmatched {{ count: {len(stack)}")
    else:
        print("Balanced")

solve()
