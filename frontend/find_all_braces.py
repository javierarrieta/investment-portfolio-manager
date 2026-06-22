import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    pattern = re.compile(r'(\".*?\"|\'.*?\'|`.*?`|//.*|/\*.*?\*/)', re.DOTALL)
    
    others = []
    for match in pattern.finditer(content):
        others.append((match.start(), match.end()))
    
    others.sort()

    braces = []
    for match in re.finditer(r'\{|\}', content):
        braces.append((match.start(), match.group()))
    
    stack = []
    for b_idx, b_char in braces:
        in_other = False
        for o_start, o_end in others:
            if o_start <= b_idx < o_end:
                in_other = True
                break
        
        if not in_other:
            if b_char == '{':
                stack.append(b_idx)
                print(f"CODE {{ at {b_idx}")
            else:
                if stack:
                    popped = stack.pop()
                    print(f"CODE }} at {b_idx}, popped {popped}")
                else:
                    print(f"EXTRA }} at {b_idx}")

    if stack:
        print(f"UNMATCHED {{: {stack}")
    else:
        print("BALANCED")

solve()
