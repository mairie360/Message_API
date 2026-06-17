module.exports = {
  mairie360: {
    input: "./openapi.json",
    output: {
      mode: "split",
      target: "generated/endpoints",
      schemas: "generated/model",
      client: "axios",
      mock: false,
      override: {
        // Force Orval à générer les fonctions sous forme de hooks/fonctions standard
        // et évite qu'il sépare les requêtes de lecture et d'écriture.
        operations: {
          create_chat: {
            client: "axios",
          },
        },
      },
    },
  },
};
