import re

def solve():
    path = 'src/components/AnalyticsView.tsx'
    with open(path, 'r') as f:
        content = f.read()

    # Use a very simple way to find all { and } and their indices
    # We'll also try to identify if they are in strings/comments
    
    # This regex matches comments and strings
    pattern = re.compile(r'(\/\*.*?\*\/|\/\/.*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|`(?:\\.|[^`\\])*`)', re.DOTALL)
    
    stack = []
    pos = 0
    
    # We'll iterate through the whole file once
    for match in pattern.finditer(content):
        # The text before the match
        text_before = content[pos:match.start()]
        for m in re.finditer(r'\{|\}', text_before):
            if m.group() == '{':
                stack.append(pos + m.start())
                print(f"Found {{ at {pos + m.start()}")
            else:
                if stack:
                    popped = stack.pop()
                    print(f"Found }} at {pos + m.start()}, popped {popped}")
                else:
                    print(f"Unmatched }} at {pos + m.start()}")
        pos = match.end()
        
    # Remaining text
    text_after = content[pos:]
    for m in re.finditer(r'\{|\}', text_after):
        if m.group() == '{':
            stack.append(pos + m.start())
            print(f"Found {{ at {pos + m.start()}")
        else:
            if stack:
                popped = stack.pop()
                print(f"Found }} at {pos + m.start()}, popped {popped}")
            else:
                print(f"Unmatched }} at {pos + m.start()}")
                
    if stack:
        print(f"Unmatched {{ left: {stack}")
    else:
        print("Balanced")

solve()
