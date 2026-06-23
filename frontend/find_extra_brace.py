import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    # Regex for strings and comments. 
    # IMPORTANT: // comment should NOT match newlines.
    # So we don't use re.DOTALL for the whole thing, 
    # or we use a better regex for //
    pattern = re.compile(r'(\/\*.*?\*\/|\/\/[^\n]*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|`(?:\\.|[^`\\])*`)', re.DOTALL)
    
    stack = []
    pos = 0
    for match in pattern.finditer(content):
        # Process text before the match
        text_before = content[pos:match.start()]
        for m in re.finditer(r'\{|\}', text_before):
            if m.group() == '{':
                stack.append(pos + m.start())
            else:
                if stack:
                    stack.pop()
                else:
                    print(f"EXTRA }} found at index {pos + m.start()}")
                    return
        pos = match.end()

    # Process remaining text
    text_after = content[pos:]
    for m in re.finditer(r'\{|\}', text_after):
        if m.group() == '{':
            stack.append(pos + m.start())
        else:
            if stack:
                stack.pop()
            else:
                print(f"EXTRA }} found at index {pos + m.start()}")
                return
    
    if stack:
        print(f"UNMATCHED {{: {stack}")
    else:
        print("BALANCED")

solve()
