/**
 * Internal Bits UI gateway.
 *
 * Feature code should consume the styled Qiongli UI components exported from
 * this directory. Keeping the unstyled primitives behind one module makes the
 * accessibility foundation replaceable without leaking library-specific APIs
 * into routes.
 */
export {
  Dialog as DialogPrimitive,
  Tabs as TabsPrimitive
} from 'bits-ui';
