export type UiTone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';
export type UiMaterial = 'solid' | 'glass' | 'glass-strong';

type ClassValue = string | false | null | undefined;

export function uiClasses(...values: ClassValue[]): string {
  return values.filter(Boolean).join(' ');
}

export function materialClass(
  material: Exclude<UiMaterial, 'solid'>,
  ...classes: ClassValue[]
): string {
  return uiClasses(
    ...classes,
    'glass-material',
    material === 'glass-strong' && 'glass-material--strong'
  );
}

export function surfaceClass(
  material: UiMaterial = 'solid',
  ...classes: ClassValue[]
): string {
  return uiClasses(
    ...classes,
    'surface',
    material !== 'solid' && 'glass-material',
    material === 'glass-strong' && 'glass-material--strong'
  );
}
