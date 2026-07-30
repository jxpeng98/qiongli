export interface AcademicGraphRequestToken {
  generation: number;
  scope: string;
}

/**
 * Rejects late read results, including an A -> B -> A switch where the stable
 * project identity alone cannot distinguish the older A request.
 */
export class AcademicGraphRequestSequence {
  private generation = 0;

  begin(scope: string): AcademicGraphRequestToken {
    this.generation += 1;
    return { generation: this.generation, scope };
  }

  invalidate(): void {
    this.generation += 1;
  }

  isCurrent(token: AcademicGraphRequestToken, activeScope: string | null): boolean {
    return token.generation === this.generation && token.scope === activeScope;
  }
}
