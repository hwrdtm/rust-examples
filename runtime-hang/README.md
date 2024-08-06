# Runtime Hangs

This example demonstrates how, when the thread is doing some blocking work and there exists a long time between `.await` boundaries then other tasks **within the same runtime** can't make progress.