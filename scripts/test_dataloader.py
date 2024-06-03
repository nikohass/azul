from azul2 import *
import time
import threading

loader = DataLoader("http://127.0.0.1:3044", 64)
loader.set_target_buffer_size(1000)

count = 0
start_time = time.time()

iter_loader = iter(loader)
while True:
    load_time = time.time()
    # try:
    next_batch = loader.__next__()
    # except Exception as e:
#        # print(f"Exception during loading: {e}")
        # break
    # print(next_batch)
    print("Time to load batch: ", time.time() - load_time)
    
    count += 1
    elapsed_time = time.time() - start_time
    print("Time per batch: ", elapsed_time / count)

    # # Check thread status
    # for t in threading.enumerate():
    #     print(f"Thread {t.name}: is alive: {t.is_alive()}")
    # print("Main loop is still running")


print("Total time: ", time.time() - start_time)
print("This is the end")
