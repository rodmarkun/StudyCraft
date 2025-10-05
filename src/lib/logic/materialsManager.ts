import type { StudyMaterial, ReviewMaterial } from '../stores/materialsStore';

export class MaterialsManager {
  private containerWidth = 0;

  calculateGridProperties(containerElement: HTMLElement | null, zoomLevel: number = 1) {
    if (containerElement) {
      this.containerWidth = containerElement.offsetWidth;
    }
    
    if (this.containerWidth === 0 && containerElement) {
      const rect = containerElement.getBoundingClientRect();
      this.containerWidth = rect.width;
    }
    const baseCardSizeRem = 16; // 16rem base card width
    const baseCardSizePx = baseCardSizeRem * 16; // Convert to pixels     
    const scaledCardSizePx = baseCardSizePx * zoomLevel;
    
    const baseGapRem = 1.5;
    const scaledGapPx = baseGapRem * 16 * zoomLevel;

    const minColumns = 1;
    const maxColumns = 12;     
    let columns = 4; // Default fallback
    
    const effectiveWidth = this.containerWidth > 0 ? this.containerWidth : 1200;     
    const containerPaddingPx = 2 * 16 * 2; // 2rem each side in pixels
    const availableWidth = effectiveWidth - containerPaddingPx;
    const cardWithGap = scaledCardSizePx + scaledGapPx;
    const possibleColumns = Math.floor((availableWidth + scaledGapPx) / cardWithGap);
    
    columns = Math.max(minColumns, Math.min(maxColumns, possibleColumns));
    
    if (columns <= 0) {
      columns = 1;
    }

    // Grid gap scales with zoom level
    const gridGap = Math.max(0.75, 1.5 * zoomLevel) + 'rem';    
    const gridStyle = `repeat(${columns}, minmax(0, 1fr))`;

    return {
      columns,
      gridGap,
      gridStyle
    };
  }

  handleResize() {
  }

  // Get identifier for a material (either file_name or id)
  getMaterialIdentifier(material: StudyMaterial | ReviewMaterial): string {
    return material.id;
  }

  // Check if a material has a specific identifier
  hasMaterialIdentifier(material: StudyMaterial | ReviewMaterial, identifier: string): boolean {
    return this.getMaterialIdentifier(material) === identifier;
  }

  // Get the display name for a material
  getMaterialName(material: StudyMaterial | ReviewMaterial): string {
    if ('file_name' in material) {
      return material.display_name || material.name;
    } else {
      return material.name;
    }
  }

  // Get a count of items (cards or questions) for a review material
  getItemsCount(material: ReviewMaterial): number {
    if (material.type === 'flashcard_deck') {
      return material.cards_count || 0;
    } else {
      return material.questions_count || 0;
    }
  }

  // Get the item type name (cards or questions)
  getItemsTypeName(material: ReviewMaterial): string {
    return material.type === 'flashcard_deck' ? 'cards' : 'questions';
  }
}