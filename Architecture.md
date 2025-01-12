## This document describes the architecture that we are working towards



## Research Questions

To avoid spinning in circles, we need to focus on demonstrating / enabling specific scenarios / flows. I think the following
would be useful:
  1. An aggregation service that is not trusted to calculate correctly, but trusted with patient data.
  2. An aggregation service that has no trust whatsoever.

I want to execute the query:
```sparql
CONSTRUCT {
    
} WHERE {

}
```

To investigate:
 - Security ontology
 - 

SPARQL SMPC:
 - https://inria.hal.science/hal-02544920v1

Graph Database SMPC:
 - https://www.scitepress.org/Papers/2022/108762/108762.pdf

## Challenges

 - oxigraph does not support inferencing https://github.com/oxigraph/oxigraph/issues/130

## Tangential directions

### SSL -> Signed RDF-star

Use [DECO: Liberating Web Data Using Decentralized Oracles for TLS](https://dl.acm.org/doi/10.1145/3372297.3417239) which is able to prove the provenance of data fetched from a certain website; i.e. allowing me to __add a signature__ when fetching data from a particular platform.
