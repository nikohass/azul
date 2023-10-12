
for row in range(32):
    # Convert first 5 bits to a list of booleans
    bits = [bool(row & (1 << i)) for i in range(5)]
    print(bits)