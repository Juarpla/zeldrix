export interface WorkflowStep {
  label: string;
  description: string;
}

export interface AutomationInputField {
  id: string;
  label: string;
  placeholder: string;
  type: "text" | "textarea" | "file";
  defaultValue?: string;
}

export interface AutomationShortcut {
  id: string;
  title: string;
  description: string;
  category: "email" | "documents" | "data-extraction" | "meetings";
  difficulty: "light" | "medium" | "heavy";
  estimatedSeconds: number;
  iconName: "email" | "minutes" | "pdf" | "reply";
  steps: WorkflowStep[];
  inputs: AutomationInputField[];
  mockOutput: string;
  isThinkingMode?: boolean;
}

export const AUTOMATIONS_DATA: AutomationShortcut[] = [
  {
    id: "summarize-email",
    title: "Resumir Correo",
    description: "Extrae de forma automática los puntos clave y compromisos urgentes de correos electrónicos corporativos extensos.",
    category: "email",
    difficulty: "light",
    estimatedSeconds: 3,
    iconName: "email",
    inputs: [
      {
        id: "emailContent",
        label: "Cuerpo del Correo Electrónico",
        placeholder: "Pega el contenido completo del correo aquí...",
        type: "textarea",
        defaultValue: `Estimado equipo,

Espero que se encuentren muy bien. Les escribo para darles seguimiento a los preparativos de la auditoría externa del próximo mes. Necesitamos consolidar todos los informes financieros antes del viernes 15 a las 18:00 horas sin excepción. 

Laura, por favor asegúrate de revisar que los balances generales cuadren al 100% y envía el reporte a auditoria@empresa.com. Sergio, tú debes preparar el inventario físico actualizado de la bodega norte y tenerlo listo para la inspección visual que se llevará a cabo el martes 12.

Cualquier duda o retraso, avísenme de inmediato para coordinar planes de contingencia. Esta auditoría es crítica para mantener nuestra certificación de calidad anual.

Saludos cordiales,
Martín Director de Finanzas`
      }
    ],
    steps: [
      { label: "Análisis preliminar", description: "Detectando estructura y participantes del correo." },
      { label: "Extracción semántica", description: "Identificando compromisos, fechas límite y responsables." },
      { label: "Generación de síntesis", description: "Redactando el resumen estructurado en viñetas ejecutivas." }
    ],
    mockOutput: `<h1>Resumen Ejecutivo: Preparativos Auditoría Externa</h1>

<p><strong>Remitente:</strong> Martín (Director de Finanzas)</p>
<p><strong>Objetivo Principal:</strong> Consolidar informes financieros e inventario para la auditoría externa.</p>

<h3>Acciones Clave y Responsabilidades:</h3>
<ul>
  <li><strong>Laura:</strong> Revisar balances generales financieros y enviar reporte a <code>auditoria@empresa.com</code>. <br/><em>Fecha Límite: Viernes 15, 18:00 hrs.</em></li>
  <li><strong>Sergio:</strong> Preparar el inventario físico actualizado de la bodega norte para la inspección visual. <br/><em>Fecha Límite: Martes 12.</em></li>
</ul>

<h3>Puntos Críticos:</h3>
<ul>
  <li>Esta auditoría determina la renovación de la certificación de calidad anual de la empresa.</li>
  <li>Se requiere reporte inmediato a Martín ante cualquier desviación o retraso detectado.</li>
</ul>`
  },
  {
    id: "structure-minutes",
    title: "Estructurar Minuta",
    description: "Transforma transcripciones desordenadas y notas rápidas de reuniones en actas corporativas pulidas y con tareas accionables.",
    category: "meetings",
    difficulty: "medium",
    estimatedSeconds: 5,
    iconName: "minutes",
    inputs: [
      {
        id: "meetingNotes",
        label: "Notas de la Reunión / Transcripción",
        placeholder: "Pega las notas de voz o el acta borrador aquí...",
        type: "textarea",
        defaultValue: `Reunión de estatus del lunes a las 9am. Asistentes: Pedro, Sofía y Carlos.
Sofía dice que el rediseño web está casi listo, pero faltan los artes finales de marketing. Pedro dice que él se encarga de hablar con el diseñador para que entregue los artes el miércoles.
Carlos comenta que los servidores de producción necesitan una actualización de seguridad urgente. Dice que lo hará este viernes a las 11 PM para no afectar a los usuarios activos.
Sofía recuerda que hay que definir el presupuesto de publicidad del próximo Q. Quedaron en reunirse de nuevo el próximo lunes a la misma hora para cerrar ese tema. Pedro armará la invitación.`
      }
    ],
    steps: [
      { label: "Análisis conversacional", description: "Identificando asistentes, agendas y turnos de habla." },
      { label: "Mapeo de decisiones", description: "Registrando acuerdos concluyentes y debates pendientes." },
      { label: "Estructuración corporativa", description: "Dando formato oficial a la minuta con tabla de accionables." }
    ],
    mockOutput: `<h1>Acta de Reunión: Estatus Semanal de Operaciones</h1>

<p><strong>Fecha y Hora:</strong> Lunes, 09:00 AM</p>
<p><strong>Participantes:</strong> Pedro, Sofía, Carlos</p>

<h2>Resumen de la Sesión</h2>
<p>Se revisó el progreso del rediseño del portal web, la planeación de la seguridad de la infraestructura y los preparativos presupuestarios del próximo trimestre.</p>

<h2>Acuerdos y Decisiones</h2>
<ol>
  <li>El lanzamiento web está condicionado a la recepción de piezas gráficas de mercadeo.</li>
  <li>Se programará el mantenimiento preventivo de infraestructura en horario no laboral para mitigar interrupciones.</li>
  <li>La planeación financiera comercial se pospone para una sesión de trabajo dedicada el siguiente lunes.</li>
</ol>

<h2>Tabla de Acciones y Compromisos</h2>
<table border="1" cellpadding="6" style="border-collapse: collapse; width: 100%; border: 1px solid #ddd;">
  <thead>
    <tr style="background-color: #f2f2f2; text-align: left;">
      <th>Tarea / Compromiso</th>
      <th>Responsable</th>
      <th>Fecha Límite</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>Coordinar con diseño la entrega final de artes publicitarios.</td>
      <td>Pedro</td>
      <td>Miércoles</td>
    </tr>
    <tr>
      <td>Actualización de seguridad en servidores de producción (11:00 PM).</td>
      <td>Carlos</td>
      <td>Viernes</td>
    </tr>
    <tr>
      <td>Configurar convocatoria para sesión presupuestal del próximo Q.</td>
      <td>Pedro</td>
      <td>Lunes</td>
    </tr>
  </tbody>
</table>`,
    isThinkingMode: true,
  },
  {
    id: "extract-pdf-table",
    title: "Extraer Tabla de PDF",
    description: "Detecta, procesa y extrae tablas tabulares complejas incrustadas en documentos PDF y las exporta a texto formateado limpio.",
    category: "data-extraction",
    difficulty: "heavy",
    estimatedSeconds: 8,
    iconName: "pdf",
    inputs: [
      {
        id: "pdfFile",
        label: "Cargar Archivo de Datos (PDF / Imagen)",
        placeholder: "Selecciona un informe en formato PDF...",
        type: "file"
      }
    ],
    steps: [
      { label: "Ingesta y OCR de documento", description: "Buscando capas estructuradas y aplicando OCR si es requerido." },
      { label: "Detección de bordes", description: "Localizando celdas de cuadrículas, encabezados de columnas y totales." },
      { label: "Extracción tabular", description: "Normalizando datos numéricos y eliminando caracteres basura." },
      { label: "Validación de coherencia", description: "Comprobando cuadres matemáticos y consistencia relacional." }
    ],
    mockOutput: `<h2>Tabla de Datos Extraída: Reporte de Rendimiento Comercial Q1</h2>
<p><strong>Origen:</strong> Documento Digital Extracted_Report_V2.pdf (Página 3)</p>

<table border="1" cellpadding="6" style="border-collapse: collapse; width: 100%; border: 1px solid #ddd;">
  <thead>
    <tr style="background-color: #f2f2f2; text-align: left;">
      <th>Código de Región</th>
      <th>Ventas Q1 (USD)</th>
      <th>Variación Anual (%)</th>
      <th>Cumplimiento Meta (%)</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>REG-NORTH-01</td>
      <td>$124,500.00</td>
      <td>+4.2%</td>
      <td>102.5%</td>
    </tr>
    <tr>
      <td>REG-SOUTH-02</td>
      <td>$98,200.00</td>
      <td>-1.8%</td>
      <td>95.4%</td>
    </tr>
    <tr>
      <td>REG-EAST-03</td>
      <td>$145,900.00</td>
      <td>+8.5%</td>
      <td>108.9%</td>
    </tr>
    <tr>
      <td>REG-WEST-04</td>
      <td>$110,400.00</td>
      <td>+0.1%</td>
      <td>99.0%</td>
    </tr>
  </tbody>
  <tfoot>
    <tr style="font-weight: bold; background-color: #f9f9f9;">
      <td>Consolidado General</td>
      <td>$479,000.00</td>
      <td>+2.75%</td>
      <td>101.45%</td>
    </tr>
  </tfoot>
</table>`
  },
  {
    id: "draft-reply",
    title: "Redactar Respuesta",
    description: "Redacta de manera autónoma un borrador formal de respuesta basado en notas breves e intenciones del usuario.",
    category: "email",
    difficulty: "light",
    estimatedSeconds: 2.5,
    iconName: "reply",
    inputs: [
      {
        id: "context",
        label: "Notas de Respuesta Rápida",
        placeholder: "Ej: Decirle que sí aceptamos la cotización, pero que necesitamos el contrato para firma mañana a primera hora.",
        type: "textarea",
        defaultValue: "Decir que recibimos el portafolio, nos gusta la propuesta del plan de mantenimiento anual, y queremos agendar llamada técnica este jueves a las 3 PM."
      }
    ],
    steps: [
      { label: "Análisis de intención", description: "Determinando el tono, cortesía y directrices principales." },
      { label: "Generación de borrador", description: "Construyendo la estructura del correo con saludos formales." },
      { label: "Revisión de claridad", description: "Verificando cohesión y llamadas a la acción claras." }
    ],
    mockOutput: `<p>Estimado Proveedor,</p>

<p>Agradecemos el envío de su portafolio de servicios y la propuesta detallada para el plan de mantenimiento anual. Hemos revisado la información preliminar y consideramos de gran valor técnico su planteamiento de soporte preventivo.</p>

<p>Con el fin de resolver inquietudes operativas menores y finiquitar los detalles técnicos de la integración, nos gustaría proponer una breve llamada de trabajo para este <strong>jueves a las 15:00 horas (hora local)</strong>.</p>

<p>Por favor, confírmenos si cuenta con disponibilidad en dicho bloque de tiempo para remitirle la invitación de sala digital correspondiente.</p>

<p>Atentamente,<br/>
<strong>Equipo de Gestión de Infraestructura</strong></p>`
  }
];
