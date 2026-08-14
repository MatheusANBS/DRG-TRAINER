export const formatNumber = (value: number) =>
  new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 0 }).format(value);

export const errorText = (error: unknown) =>
  typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
