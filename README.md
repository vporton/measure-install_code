# `measure`

```
dfx deploy
dfx generate
```

To measure, use some real big modules, like:
```sh
npx tsx measure.ts ~/Projects/icp-package-manager/.dfx/local/canisters/bootstrapper/bootstrapper.wasm ~/Projects/icp-package-manager/.dfx/local/canisters/internet_identity/internet_identity.wasm.gz
```
(from https://github.com/vporton/icp-package-manager).

The output was:

```
$ cat measures.log 
858593 912304421
2005985 2122803981
```

Then approximate it with GNU Plot:

```
$ gnuplot
gnuplot> f(x) = m*x + b
gnuplot> fit f(x) 'measures.log' via m, b
...
m               = 1058.9
b               = 2.52313
```
