import { invoke } from "@tauri-apps/api/core";
import type { Template } from "./types";

// Mock data for when Tauri backend is not available
const MOCK_TEMPLATES: Template[] = [
  {
    id: 1,
    name: "Carta de Presentación Corporativa",
    category: "ventas",
    required_variables: ["nombre_cliente", "empresa", "fecha", "monto_propuesta"],
    system_prompt: "Eres un asistente de redacción corporativa. Genera cartas formales manteniendo el tono profesional de la empresa.",
    base_text: `Estimado/a {{nombre_cliente}}:

Por medio de la presente, {{empresa}} se complace en presentar la propuesta por un monto de {{monto_propuesta}}, con fecha {{fecha}}.

Quedamos a su disposición para cualquier consulta.

Atentamente,
Equipo Corporativo`,
  },
  {
    id: 2,
    name: "Contrato de Prestación de Servicios",
    category: "legal",
    required_variables: ["nombre_proveedor", "servicio", "duracion", "monto_mensual"],
    system_prompt: "Eres un abogado corporativo especializado en contratos de servicios. Redacta contratos claros y legalmente sound.",
    base_text: `CONTRATO DE PRESTACIÓN DE SERVICIOS

Entre las partes:
Contratante: Empresa XYZ
Prestador: {{nombre_proveedor}}

Se acordo prestar el servicio de {{servicio}} por un período de {{duracion}}, con un monto mensual de {{monto_mensual}}.

CLAUSULAS:
1. El prestador se compromete a...
2. El contratante se compromete a...
3. Las partes pueden rescindir con 30 días de anticipación.`,
  },
  {
    id: 3,
    name: "Convocatoria de Entrevista",
    category: "recursos humanos",
    required_variables: ["nombre_candidato", "puesto", "fecha_entrevista", "hora", "lugar"],
    system_prompt: "Eres un analista de recursos humanos. Redacta convocatorias profesionales y cordiales.",
    base_text: `Estimado {{nombre_candidato}}:

Nos complace informarle que su perfil ha sido seleccionado para el puesto de {{puesto}}.

Fecha: {{fecha_entrevista}}
Hora: {{hora}}
Lugar: {{lugar}}

Por favor confirme su asistencia respondiendo a este correo.

Saludos cordiales,
Equipo de Recursos Humanos`,
  },
  {
    id: 4,
    name: "Acuerdo de Confidencialidad (NDA)",
    category: "legal",
    required_variables: ["nombre_partre", "empresa_receptora", "fecha_inicio", "duracion_meses"],
    system_prompt: "Eres un abogado especializado en propiedad intelectual. Redacta NDAs estándar de la industria.",
    base_text: `ACUERDO DE CONFIDENCIALIDAD

Entre {{empresa_receptora}} ("Parte Receptora") y la Empresa ("Parte Divulgadora")

Fecha de inicio: {{fecha_inicio}}
Duración: {{duracion_meses}} meses

La Parte Receptora se compromete a no divulgar información confidencial de la Parte Divulgadora.

Este acuerdo entra en vigor a partir de la fecha de firma.`,
  },
  {
    id: 5,
    name: "Propuesta Comercial",
    category: "ventas",
    required_variables: ["nombre_empresa", "servicio_ofrecido", "beneficios_clave", "precio", "validez"],
    system_prompt: "Eres un especialista en ventas B2B. Crea propuestas convincentes que destacan valor.",
    base_text: `PROPUESTA COMERCIAL

Para: {{nombre_empresa}}

Servicio: {{servicio_ofrecido}}

BENEFICIOS CLAVE:
{{beneficios_clave}}

INVERSIÓN: {{precio}}

Esta propuesta tiene validez por {{validez}}.

Quedamos a su disposición para negociar los términos.`,
  },
  {
    id: 6,
    name: "Comunicado de Cambio de Política",
    category: "recursos humanos",
    required_variables: ["titulo_politica", "fecha_efectiva", "cambiosprincipales", "contacto_rh"],
    system_prompt: "Eres un experto en comunicación interna corporativa. Redacta comunicados claros y empáticos.",
    base_text: `COMUNICADO INTERNO

Asunto: Actualización de Política - {{titulo_politica}}

Fecha efectiva: {{fecha_efectiva}}

Cambios principales:
{{cambiosprincipales}}

Si tiene preguntas, contacte a {{contacto_rh}}.

Agradecemos su comprensión,
Dirección`,
  },
];

let useMockData = true;

export async function getTemplates(): Promise<Template[]> {
  if (useMockData) {
    // Simulate network delay for skeleton demo
    await new Promise((resolve) => setTimeout(resolve, 800));
    return MOCK_TEMPLATES;
  }

  try {
    return await invoke<Template[]>("template_list");
  } catch (e) {
    console.warn("Tauri command 'template_list' not available, using mock data:", e);
    useMockData = true;
    return MOCK_TEMPLATES;
  }
}

export async function getTemplateById(id: number): Promise<Template | null> {
  if (useMockData) {
    await new Promise((resolve) => setTimeout(resolve, 600));
    return MOCK_TEMPLATES.find((t) => t.id === id) || null;
  }

  try {
    return await invoke<Template>("template_get", { id });
  } catch (e) {
    console.warn("Tauri command 'template_get' not available, using mock data:", e);
    return MOCK_TEMPLATES.find((t) => t.id === id) || null;
  }
}