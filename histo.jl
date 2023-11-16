using Plots

file = open("out.txt")
data = readlines(file)
data = map(x->parse(Int,x),data)

histogram(data, size=(2000, 1000), minorticks=5)