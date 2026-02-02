
* **ANNS + p95/max NN stretch (space→curve locality)**
  Measures how well *spatial neighbors* stay close in the curve’s 1D order. For
  each grid point and each L1-adjacent neighbor, compute **stretch = |index(p)
  − index(q)|**. Report the mean (ANNS), **p95**, and **max** stretch. Lower is
  better; a large p95/max means the curve “tears” neighborhoods apart.

* **WL∞ / WL2 profiles over segment lengths (curve→space locality)**
  Measures how compact a *contiguous run of indices* is in space. For each
  segment length **L**, slide a window of **L** consecutive indices, compute
  the **endpoint distance** (L∞ or L2), normalize as **(dist^d)/L**, and take
  the worst case (and optionally distribution stats). Plot results vs **L**.
  Lower is better; spikes identify block sizes that sprawl across space.
