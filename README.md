# 🧱 SolDB Program

**SolDB** es un programa on-chain escrito en **Rust** para la blockchain de **Solana**.  
Su objetivo es proporcionar una base de datos simple basada en tablas y pares clave-valor, completamente gestionada dentro de cuentas PDA del propio programa.

---

## 📘 Descripción general

El programa implementa una arquitectura de almacenamiento jerárquica donde:

- Cada **tabla** es una cuenta PDA derivada del nombre de la tabla y la clave pública del creador.
- Cada **valor** se almacena en una cuenta PDA derivada de la tabla y la clave del registro.
- Los datos se serializan utilizando **Borsh** para optimizar espacio y compatibilidad.

El programa incluye operaciones básicas:
- **InitTable** → Crea una nueva tabla.
- **Put** → Inserta o actualiza un par clave-valor dentro de una tabla existente.
- **Delete** → Elimina un registro específico de una tabla.
- (Opcionalmente: futuras operaciones de lectura o limpieza masiva).

---

## ⚙️ Estructura del proyecto

```
soldb_program/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── entrypoint.rs
│   ├── processor.rs
│   ├── instructions.rs
│   ├── accounts.rs
│   ├── error.rs
│   └── utils.rs
└── tests/
    ├── init_table_tests.rs
    ├── put_tests.rs
    ├── delete_tests.rs
    └── utils.rs
```

- **instructions.rs** → Define las instrucciones (`InitTable`, `Put`, `Delete`).
- **processor.rs** → Implementa la lógica principal de cada instrucción.
- **accounts.rs** → Define las estructuras `SolTable` y `SolValue`.
- **error.rs** → Enumera errores personalizados (`SolDbError`).
- **tests/** → Pruebas de integración con `solana-program-test`.

---

## 🧰 Tecnologías utilizadas

- **Rust** (con nightly features para Solana)
- **Solana SDK** `v2.x`
- **Borsh** para serialización binaria
- **Program Test Framework** para tests locales
- **Cargo Make** / **Makefile** para simplificar builds

---

## 🚀 Compilación e instalación

1. **Construir el programa (BPF/SBF)**  
   ```bash
   cargo build-sbf
   ```

2. **Ejecutar tests**  
   ```bash
   cargo test-sbf
   ```

3. **Opcional: ejecutar pruebas unitarias nativas**
   ```bash
   cargo test
   ```

---

## 🧪 Ejemplo de instrucción

```rust
let instr = SolDbIntructions::InitTable(InitTable {
    name: "Users".to_string(),
    bump,
});
```

Cada instrucción se serializa con `borsh` y se envía como parte de una `Transaction` de Solana.  
Las cuentas PDA se derivan de los seeds `["table_name", owner_pubkey]`.

---

## 🧩 Diseño del almacenamiento

```text
┌────────────────────────┐
│ Owner Account (signer) │
└────────────┬───────────┘
             │
             ▼
      ┌─────────────┐
      │  Table PDA  │  ← derived from (table_name, owner)
      └─────────────┘
             │
             ▼
      ┌─────────────┐
      │  Value PDA  │  ← derived from (key, table_pda, owner)
      └─────────────┘
```

Cada PDA contiene datos serializados en formato Borsh:  
- `SolTable { name: String }`  
- `SolValue { val: Vec<u8> }`

---

## 📄 Licencia

Este proyecto se distribuye bajo la licencia **MIT**.  
Consulta el archivo [`LICENSE`](./LICENSE) para más detalles.

---

## ✍️ Autor

**Pedro Llinás Ferrer**  
Desarrollador de SolDB – Universidad Politécnica de Madrid (ETSISI)

---

## 🌐 Próximos pasos

- Añadir operaciones de **Get/Scan** para lectura directa.  
- Soporte para **resizing dinámico** de cuentas PDA.  
- Integración con cliente off-chain en Rust o TypeScript.
