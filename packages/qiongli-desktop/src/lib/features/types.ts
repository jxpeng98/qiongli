export type FeatureStage = 'available' | 'r4a' | 'r4b' | 'r5c';

export interface FeatureDescriptor {
  id: string;
  label: string;
  route: string | null;
  stage: FeatureStage;
}
