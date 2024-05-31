from azul2 import *
import time

loader = DataLoader("http://127.0.0.1:3044", 64)
loader.set_target_buffer_size(100)

for batch in loader:
    print(batch)
    time.sleep(0.1)