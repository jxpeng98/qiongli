<script lang="ts">
	import { AlertDialog as AlertDialogPrimitive } from "bits-ui";
	import { cn, type WithoutChild, type WithoutChildrenOrChild } from "$lib/utils.js";
	import AlertDialogOverlay from "./alert-dialog-overlay.svelte";
	import AlertDialogPortal from "./alert-dialog-portal.svelte";
	import type { ComponentProps } from "svelte";

	let {
		ref = $bindable(null),
		class: className,
		size = "default",
		portalProps,
		overlayProps,
		...restProps
	}: WithoutChild<AlertDialogPrimitive.ContentProps> & {
		size?: "default" | "sm";
		portalProps?: WithoutChildrenOrChild<ComponentProps<typeof AlertDialogPortal>>;
		overlayProps?: WithoutChildrenOrChild<ComponentProps<typeof AlertDialogOverlay>>;
	} = $props();
</script>

<AlertDialogPortal {...portalProps}>
	<AlertDialogOverlay {...overlayProps} />
	<AlertDialogPrimitive.Content
		bind:ref
		data-slot="alert-dialog-content"
		data-size={size}
		class={cn(
			"gap-6 rounded-[min(var(--radius-4xl),24px)] bg-popover p-6 text-popover-foreground shadow-xl ring-1 ring-foreground/5 duration-100 data-[size=default]:max-w-xs data-[size=sm]:max-w-xs data-[size=default]:sm:max-w-md dark:ring-foreground/10 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95 group/alert-dialog-content fixed top-1/2 left-1/2 z-50 grid w-full -translate-x-1/2 -translate-y-1/2 outline-none",
			className
		)}
		{...restProps}
	/>
</AlertDialogPortal>
