cflake_engine::main:

		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 14
		fn main() {
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 792
	lea rbp, [rsp + 128]
	movdqa xmmword ptr [rbp + 640], xmm6
	mov qword ptr [rbp + 632], -2
	lea rcx, [rbp - 72]

		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 29
		let mut scene = Scene::default();
	call <ecs::scene::Scene as core::default::Default>::default

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 16
		let masks = [$(mask::<$name>()),+];

	call ecs::registry::mask

	mov rsi, rax
	call ecs::registry::mask

	mov rbx, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 24
		a | b

	mov rdi, rax

	mov rcx, rsi
	mov rdx, rbx
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov rcx, rax
	mov rdx, rdi
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 22
		let mask = Self::reduce(|a, b| {
	mov qword ptr [rbp + 464], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 26
		mask.count_ones() == count as u32

	lea rcx, [rbp + 464]

	call ecs::mask::Mask::count_ones

	cmp eax, 3

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\scene.rs : 69
		assert!(
	jne .LBB77_30

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 16
		let masks = [$(mask::<$name>()),+];

	call ecs::registry::mask

	mov rsi, rax
	call ecs::registry::mask

	mov rbx, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\scene.rs : 75
		let mask = B::reduce(|a, b| a | b);

	mov rdi, rax

	mov rcx, rsi
	mov rdx, rbx
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov rcx, rax
	mov rdx, rdi
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1295
	mov rdi, rax

	mov rbx, rax

	mov rax, qword ptr [rbp - 40]
	mov rcx, qword ptr [rbp - 32]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov r14, rdi
	shr r14, 57
	movd xmm0, r14d
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	lea rdx, [rax - 72]
	xor r8d, r8d
	pcmpeqd xmm1, xmm1
	mov r9, rdi

.LBB77_13:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and r9, rcx

	movdqu xmm2, xmmword ptr [rax + r9]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r10d, xmm3

.LBB77_14:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r10w, r10w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_15

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea esi, [r10 - 1]
	and esi, r10d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r10d, r10d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r10, r9

	and r10, rcx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r10

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	lea r11, [r10 + 8*r10]

	mov r10d, esi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [rdx + 8*r11], rdi

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_14
	jmp .LBB77_33

.LBB77_15:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r10d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r10d, r10d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_17

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add r9, r8

	add r9, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add r8, 16

	jmp .LBB77_13

.LBB77_33:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1300
	lea r9, [rax + 8*r11]

.LBB77_105:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 79
		assert_eq!(self.mask(), B::reduce(|a, b| a | b));
	mov qword ptr [rbp + 616], r9

	lea rsi, [r9 - 64]

	mov rcx, rsi
	call ecs::archetype::archetype::Archetype::mask

	mov r12, rax
	mov qword ptr [rbp + 288], rax
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 16
		let masks = [$(mask::<$name>()),+];

	call ecs::registry::mask

	mov rbx, rax
	call ecs::registry::mask

	mov r15, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 79
		assert_eq!(self.mask(), B::reduce(|a, b| a | b));

	mov r14, rax

	mov rcx, rbx
	mov rdx, r15
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov rcx, rax
	mov rdx, r14
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov qword ptr [rbp + 384], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 21
		#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
	cmp r12, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 79
		assert_eq!(self.mask(), B::reduce(|a, b| a | b));
	jne .LBB77_134

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 16
		let masks = [$(mask::<$name>()),+];

	call ecs::registry::mask

	mov r15, rax
	call ecs::registry::mask

	mov r14, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 24
		a | b

	mov rbx, rax

	mov rcx, r15
	mov rdx, r14
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov rcx, rax
	mov rdx, rbx
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 22
		let mask = Self::reduce(|a, b| {
	mov qword ptr [rbp + 464], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 26
		mask.count_ones() == count as u32

	lea rcx, [rbp + 464]

	call ecs::mask::Mask::count_ones

	cmp eax, 3

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 80
		assert!(
	jne .LBB77_201

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 88
		self.reserve(upper.unwrap_or(lower));

	mov edx, 100000
	mov rcx, rsi
	call ecs::archetype::archetype::Archetype::reserve

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rax, qword ptr [rbp + 616]

	mov rax, qword ptr [rax - 16]

	mov qword ptr [rbp + 584], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 16
		let masks = [$(mask::<$name>()),+];

	call ecs::registry::mask

	mov rsi, rax

	call ecs::registry::mask

	mov r14, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 24
		a | b

	mov rbx, rax

	mov rcx, rsi
	mov rdx, r14
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov rcx, rax
	mov rdx, rbx
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 22
		let mask = Self::reduce(|a, b| {
	mov qword ptr [rbp + 464], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\bundle.rs : 26
		mask.count_ones() == count as u32

	lea rcx, [rbp + 464]

	call ecs::mask::Mask::count_ones

	cmp eax, 3

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 25
		assert!(Self::is_valid());
	jne .LBB77_141

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 219
		self.table.get_mut(&mask::<T>())

	call ecs::registry::mask

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 1461
	mov rcx, qword ptr [rbp + 616]

	cmp qword ptr [rcx - 40], 0
	je .LBB77_202

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 1465
	mov rcx, qword ptr [rbp + 616]
	lea rdx, [rcx - 32]

	mov qword ptr [rbp + 376], rdx

	mov r15, qword ptr [rcx - 64]
	mov r12, qword ptr [rcx - 56]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, rax
	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	lea rsi, [r15 - 88]
	xor ecx, ecx
	pcmpeqd xmm1, xmm1
	mov rdx, rax

.LBB77_130:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and rdx, r12

	movdqu xmm2, xmmword ptr [r15 + rdx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r8d, xmm3

.LBB77_131:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r8w, r8w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_132

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r9d, [r8 - 1]
	and r9d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r8, rdx

	and r8, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul rdi, r8, 88

	mov r8d, r9d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [rsi + rdi], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_131
	jmp .LBB77_136

.LBB77_132:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r8d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_202

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add rdx, rcx

	add rdx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add rcx, 16

	jmp .LBB77_130

.LBB77_136:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rcx, qword ptr [r15 + rdi - 80]
	mov rax, qword ptr [r15 + rdi - 72]

	call qword ptr [rax + 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov r14, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();
	mov rcx, rax
	call qword ptr [rdx + 24]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test r14, r14

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	je .LBB77_140

	movabs rcx, -1581083357955820586
	xor rax, rcx
	movabs rcx, -7394674131754425909
	xor rdx, rcx
	or rdx, rax
	jne .LBB77_140

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 219
		self.table.get_mut(&mask::<T>())

	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 27
		let (components_C~N, delta_frame_states_C~N, delta_tick_states_C~N) = archetype.column_mut::<C~N>()?;
	lea rbx, [r15 + rdi]
	add rbx, -80
	lea rcx, [rbx + 16]

	mov qword ptr [rbp + 608], rcx
	add rbx, 48

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, rax

	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	xor ecx, ecx
	pcmpeqd xmm1, xmm1
	mov rdx, rax

.LBB77_144:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and rdx, r12

	movdqu xmm2, xmmword ptr [r15 + rdx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r8d, xmm3

.LBB77_145:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r8w, r8w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_146

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r9d, [r8 - 1]
	and r9d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r8, rdx

	and r8, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul rdi, r8, 88

	mov r8d, r9d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [rsi + rdi], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_145
	jmp .LBB77_149

.LBB77_146:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r8d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_202

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add rdx, rcx

	add rdx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add rcx, 16

	jmp .LBB77_144

.LBB77_149:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rcx, qword ptr [r15 + rdi - 80]
	mov rax, qword ptr [r15 + rdi - 72]

	call qword ptr [rax + 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov r13, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();
	mov rcx, rax
	call qword ptr [rdx + 24]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test r13, r13

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	je .LBB77_153

	movabs rcx, 8457430945661996516
	xor rax, rcx
	movabs rcx, 575874667958237832
	xor rdx, rcx
	or rdx, rax
	jne .LBB77_153

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 219
		self.table.get_mut(&mask::<T>())

	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 27
		let (components_C~N, delta_frame_states_C~N, delta_tick_states_C~N) = archetype.column_mut::<C~N>()?;
	lea rcx, [r15 + rdi]
	add rcx, -80
	lea rdx, [rcx + 16]

	mov qword ptr [rbp + 600], rdx
	add rcx, 48

	mov qword ptr [rbp + 592], rcx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, rax

	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	xor ecx, ecx
	pcmpeqd xmm1, xmm1
	mov rdx, rax

.LBB77_156:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and rdx, r12

	movdqu xmm2, xmmword ptr [r15 + rdx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r8d, xmm3

.LBB77_157:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r8w, r8w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_158

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r9d, [r8 - 1]
	and r9d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r8, rdx

	and r8, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul rdi, r8, 88

	mov r8d, r9d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [rsi + rdi], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_157
	jmp .LBB77_161

.LBB77_158:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r8d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_202

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add rdx, rcx

	add rdx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add rcx, 16

	jmp .LBB77_156

.LBB77_161:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rcx, qword ptr [r15 + rdi - 80]
	mov rax, qword ptr [r15 + rdi - 72]

	call qword ptr [rax + 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov r12, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();
	mov rcx, rax
	call qword ptr [rdx + 24]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test r12, r12

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	je .LBB77_165

	movabs rcx, -5896397890751364543
	xor rax, rcx
	movabs rcx, 4724347646448115692
	xor rdx, rcx
	or rdx, rax
	jne .LBB77_165

	add r15, rdi
	add r15, -80
	xor esi, esi
	jmp .LBB77_186

.LBB77_195:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rax, qword ptr [r12]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov dword ptr [rax + 4*rdx], esi

	inc esi

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1834
		self.len += 1;
	inc rdx
	mov qword ptr [r12 + 16], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\cmp.rs : 1363
		fn lt(&self, other: &$t) -> bool { (*self) < (*other) }
	cmp esi, 100000

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\range.rs : 621
		if self.start < self.end {
	je .LBB77_166

.LBB77_186:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1828
		if self.len == self.buf.capacity() {
	mov rdx, qword ptr [r14 + 16]
	cmp rdx, qword ptr [r14 + 8]
	jne .LBB77_189

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1829
		self.buf.reserve_for_push(self.len);

	mov rcx, r14
	call alloc::raw_vec::RawVec<T,A>::reserve_for_push

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1832
		let end = self.as_mut_ptr().add(self.len);
	mov rdx, qword ptr [r14 + 16]

.LBB77_189:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rax, qword ptr [r14]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov dword ptr [rax + 4*rdx], esi

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1834
		self.len += 1;
	inc rdx
	mov qword ptr [r14 + 16], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1828
		if self.len == self.buf.capacity() {
	mov rdx, qword ptr [r13 + 16]
	cmp rdx, qword ptr [r13 + 8]
	jne .LBB77_192

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1829
		self.buf.reserve_for_push(self.len);

	mov rcx, r13
	call alloc::raw_vec::RawVec<T,A>::reserve_for_push

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1832
		let end = self.as_mut_ptr().add(self.len);
	mov rdx, qword ptr [r13 + 16]

.LBB77_192:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rax, qword ptr [r13]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov dword ptr [rax + 4*rdx], esi

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1834
		self.len += 1;
	inc rdx
	mov qword ptr [r13 + 16], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1828
		if self.len == self.buf.capacity() {
	mov rdx, qword ptr [r12 + 16]
	cmp rdx, qword ptr [r12 + 8]
	jne .LBB77_195

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1829
		self.buf.reserve_for_push(self.len);

	mov rcx, r12
	call alloc::raw_vec::RawVec<T,A>::reserve_for_push

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1832
		let end = self.as_mut_ptr().add(self.len);
	mov rdx, qword ptr [r12 + 16]
	jmp .LBB77_195

.LBB77_166:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 54
		column~N.1.extend_with_flags(additional, StateFlags {

	mov edx, 100000
	mov rcx, qword ptr [rbp + 608]
	mov r8b, 1
	mov r9b, 1
	call ecs::archetype::states::StateColumn::extend_with_flags

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 58
		column~N.2.extend_with_flags(additional, StateFlags {

	mov edx, 100000
	mov rcx, rbx
	mov r8b, 1
	mov r9b, 1
	call ecs::archetype::states::StateColumn::extend_with_flags

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rsi, qword ptr [r14 + 16]
	mov qword ptr [rbp + 288], rsi
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 62
		assert_eq!(column~N.0.len(), column~N.1.len());

	mov rcx, qword ptr [rbp + 608]

	call ecs::archetype::states::StateColumn::len

	mov qword ptr [rbp + 384], rax

	cmp rsi, rax
	jne .LBB77_196

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rsi, qword ptr [r14 + 16]
	mov qword ptr [rbp + 288], rsi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 63
		assert_eq!(column~N.0.len(), column~N.2.len());

	mov rcx, rbx
	call ecs::archetype::states::StateColumn::len

	mov qword ptr [rbp + 384], rax

	cmp rsi, rax
	jne .LBB77_197

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 54
		column~N.1.extend_with_flags(additional, StateFlags {

	mov edx, 100000
	mov rcx, qword ptr [rbp + 600]
	mov r8b, 1
	mov r9b, 1
	call ecs::archetype::states::StateColumn::extend_with_flags

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 58
		column~N.2.extend_with_flags(additional, StateFlags {

	mov edx, 100000
	mov rcx, qword ptr [rbp + 592]
	mov r8b, 1
	mov r9b, 1
	call ecs::archetype::states::StateColumn::extend_with_flags

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rsi, qword ptr [r13 + 16]
	mov qword ptr [rbp + 288], rsi
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 62
		assert_eq!(column~N.0.len(), column~N.1.len());

	mov rcx, qword ptr [rbp + 600]

	call ecs::archetype::states::StateColumn::len

	mov qword ptr [rbp + 384], rax

	cmp rsi, rax
	jne .LBB77_198

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rsi, qword ptr [r13 + 16]
	mov qword ptr [rbp + 288], rsi
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 63
		assert_eq!(column~N.0.len(), column~N.2.len());

	mov rcx, qword ptr [rbp + 592]

	call ecs::archetype::states::StateColumn::len

	mov qword ptr [rbp + 384], rax

	cmp rsi, rax
	jne .LBB77_199

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 54
		column~N.1.extend_with_flags(additional, StateFlags {

	lea rsi, [r15 + 16]

	mov edx, 100000
	mov rcx, rsi
	mov r8b, 1
	mov r9b, 1
	call ecs::archetype::states::StateColumn::extend_with_flags

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 58
		column~N.2.extend_with_flags(additional, StateFlags {

	add r15, 48

	mov edx, 100000
	mov rcx, r15
	mov r8b, 1
	mov r9b, 1
	call ecs::archetype::states::StateColumn::extend_with_flags

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rdi, qword ptr [r12 + 16]
	mov qword ptr [rbp + 288], rdi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 62
		assert_eq!(column~N.0.len(), column~N.1.len());

	mov rcx, rsi
	call ecs::archetype::states::StateColumn::len

	mov qword ptr [rbp + 384], rax

	cmp rdi, rax
	jne .LBB77_200

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2051
		self.len
	mov rsi, qword ptr [r12 + 16]

	mov qword ptr [rbp + 288], rsi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 63
		assert_eq!(column~N.0.len(), column~N.2.len());

	mov rcx, r15
	call ecs::archetype::states::StateColumn::len

	mov qword ptr [rbp + 384], rax

	cmp rsi, rax
	jne .LBB77_184

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 934
		Some(val) => val,
	mov qword ptr [rbp + 288], 100000

	mov rax, qword ptr [rbp + 616]
	add rax, -8

	mov qword ptr [rbp + 592], rax

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 354
		let new_num_elems = self.num_elems + 1;
	mov ebx, dword ptr [rbp - 44]
	xor esi, esi
	jmp .LBB77_204

.LBB77_218:
	mov rcx, qword ptr [rbp + 616]

.LBB77_221:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	inc rsi

	mov rax, qword ptr [rcx - 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov dword ptr [rax + 8*rdx], r14d
	mov dword ptr [rax + 8*rdx + 4], r15d

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1834
		self.len += 1;
	inc rdx
	mov qword ptr [rcx - 16], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\cmp.rs : 1411
		ord_impl! { char usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }
	cmp rsi, 100000

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\range.rs : 621
		if self.start < self.end {
	je .LBB77_222

.LBB77_204:
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 354
		let new_num_elems = self.num_elems + 1;
	inc ebx

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 355
		if new_num_elems == core::u32::MAX {
	cmp ebx, -1
	je .LBB77_210

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\index.rs : 226
		if self < slice.len() { unsafe { Some(&mut *self.get_unchecked_mut(slice)) } } else { None }
	mov rax, qword ptr [rbp + 592]
	mov rax, qword ptr [rax]
	mov qword ptr [rbp + 600], rax
	mov rax, qword ptr [rbp + 584]
	add rax, rsi

	mov qword ptr [rbp + 608], rax

	mov rdi, qword ptr [rbp - 72]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2618
		unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
	mov r14, qword ptr [rbp - 56]

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 359
		if let Some(slot) = self.slots.get_mut(self.free_head as usize) {
	mov ecx, dword ptr [rbp - 48]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\index.rs : 226
		if self < slice.len() { unsafe { Some(&mut *self.get_unchecked_mut(slice)) } } else { None }
	cmp r14, rcx

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 359
		if let Some(slot) = self.slots.get_mut(self.free_head as usize) {
	jbe .LBB77_211

	test rdi, rdi
	je .LBB77_211

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\index.rs : 226
		if self < slice.len() { unsafe { Some(&mut *self.get_unchecked_mut(slice)) } } else { None }
	lea r13, [rcx + 2*rcx]

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 360
		let occupied_version = slot.version | 1;
	mov r14d, dword ptr [rdi + 8*r13 + 16]

	or r14d, 1

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 361
		let kd = KeyData::new(self.free_head, occupied_version);

	mov edx, r14d
	call slotmap::KeyData::new

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\convert\mod.rs : 716
		U::from(self)

	mov r15d, eax
	mov r12d, edx

	mov ecx, eax
	call <ecs::entity::entity::Entity as core::convert::From<slotmap::KeyData>>::from

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 368
		self.free_head = slot.u.next_free;
	lea rax, [rdi + 8*r13]

	mov ecx, dword ptr [rax]
	mov dword ptr [rbp - 48], ecx
	mov rcx, qword ptr [rbp + 600]
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 369
		slot.u.value = ManuallyDrop::new(value);
	mov qword ptr [rax], rcx
	mov rcx, qword ptr [rbp + 608]
	mov qword ptr [rax + 8], rcx
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 370
		slot.version = occupied_version;
	mov dword ptr [rax + 16], r14d
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 372
		self.num_elems = new_num_elems;
	mov dword ptr [rbp - 44], ebx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\convert\mod.rs : 716
		U::from(self)

	mov ecx, r15d
	mov edx, r12d
	call <ecs::entity::entity::Entity as core::convert::From<slotmap::KeyData>>::from

	jmp .LBB77_217

.LBB77_211:
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 377
		let kd = KeyData::new(self.slots.len() as u32, version);

	mov ecx, r14d
	mov edx, 1
	call slotmap::KeyData::new

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\convert\mod.rs : 716
		U::from(self)

	mov r15d, eax
	mov r12d, edx

	mov ecx, eax
	call <ecs::entity::entity::Entity as core::convert::From<slotmap::KeyData>>::from

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1828
		if self.len == self.buf.capacity() {
	cmp r14, qword ptr [rbp - 64]
	jne .LBB77_216

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1829
		self.buf.reserve_for_push(self.len);

	lea rcx, [rbp - 72]
	mov rdx, r14
	call alloc::raw_vec::RawVec<T,A>::reserve_for_push

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rdi, qword ptr [rbp - 72]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1832
		let end = self.as_mut_ptr().add(self.len);
	mov r14, qword ptr [rbp - 56]

.LBB77_216:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 1020
		unsafe { intrinsics::offset(self, count) }
	lea rax, [r14 + 2*r14]

	mov rcx, qword ptr [rbp + 600]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov qword ptr [rdi + 8*rax], rcx
	mov rcx, qword ptr [rbp + 608]
	mov qword ptr [rdi + 8*rax + 8], rcx
	mov dword ptr [rdi + 8*rax + 16], 1

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1834
		self.len += 1;
	inc r14

	mov qword ptr [rbp - 56], r14

		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 387
		self.free_head = kd.idx + 1;
	mov eax, r12d
	inc eax
	mov dword ptr [rbp - 48], eax
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 388
		self.num_elems = new_num_elems;
	mov dword ptr [rbp - 44], ebx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\convert\mod.rs : 716
		U::from(self)

	mov ecx, r15d
	mov edx, r12d
	call <ecs::entity::entity::Entity as core::convert::From<slotmap::KeyData>>::from

.LBB77_217:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1828
		if self.len == self.buf.capacity() {
	mov r14d, eax
	mov r15d, edx

	mov rax, qword ptr [rbp + 616]

	mov rdx, qword ptr [rax - 16]
	cmp rdx, qword ptr [rax - 24]
	jne .LBB77_218

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1829
		self.buf.reserve_for_push(self.len);

	mov rcx, qword ptr [rbp + 376]
	call alloc::raw_vec::RawVec<T,A>::reserve_for_push

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1832
		let end = self.as_mut_ptr().add(self.len);
	mov rcx, qword ptr [rbp + 616]
	mov rdx, qword ptr [rcx - 16]
	jmp .LBB77_221

.LBB77_222:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\sync\atomic.rs : 3153
		Relaxed => intrinsics::atomic_load_relaxed(dst),
	mov rax, qword ptr [rip + __imp__ZN3log20MAX_LOG_LEVEL_FILTER17he3bdb6ed9dc9e174E]

	mov rax, qword ptr [rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\cmp.rs : 1411
		ord_impl! { char usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }
	xor ecx, ecx
	cmp rax, 4
	setne cl

	mov eax, 255
	cmovbe eax, ecx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\cmp.rs : 1085
		matches!(self.partial_cmp(other), Some(Less | Equal))
	cmp al, -1
	je .LBB77_224

	movzx eax, al
	test eax, eax
	jne .LBB77_226

.LBB77_224:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 104
		log::debug!(
	mov rax, qword ptr [rbp + 592]
	mov qword ptr [rbp + 384], rax
	lea rax, [rip + <ecs::mask::Mask as core::fmt::Display>::fmt]
	mov qword ptr [rbp + 392], rax
	lea rax, [rbp + 288]
	mov qword ptr [rbp + 400], rax
	lea rax, [rip + core::fmt::num::imp::<impl core::fmt::Display for usize>::fmt]
	mov qword ptr [rbp + 408], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\fmt\mod.rs : 311
		Arguments { pieces, fmt: None, args }
	lea rax, [rip + __unnamed_25]

	mov qword ptr [rbp + 464], rax
	mov qword ptr [rbp + 472], 3
	mov qword ptr [rbp + 496], 0
	lea rax, [rbp + 384]

	mov qword ptr [rbp + 480], rax
	mov qword ptr [rbp + 488], 2

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 104
		log::debug!(

	mov qword ptr [rsp + 32], 0
	lea r8, [rip + __unnamed_26]
	lea rcx, [rbp + 464]
	mov edx, 4
	mov r9d, 104
	call log::__private_api::log

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 111
		&self.entities[old_len..]
	mov rax, qword ptr [rbp + 616]
	mov rdx, qword ptr [rax - 16]

.LBB77_226:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\index.rs : 496
		if self.start > slice.len() {
	mov rcx, qword ptr [rbp + 584]

	cmp rdx, rcx
	jb .LBB77_227

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 225
		arch_search: mask::<T>(),

	call ecs::registry::mask
	mov qword ptr [rbp + 616], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 226
		validation_shared: Mask::zero(),

	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 227
		validation_unique: mask::<T>(),

	mov rdi, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov rbx, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 168
		validation_shared: mask::<T>(),

	mov r14, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 169
		validation_unique: Mask::zero(),

	mov r15, rax
	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov r12, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 168
		validation_shared: mask::<T>(),

	mov r13, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 169
		validation_unique: Mask::zero(),

	mov rsi, rax
	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov rcx, qword ptr [rbp + 616]
	mov qword ptr [rbp + 464], rcx
	mov qword ptr [rbp + 472], rdi
	mov qword ptr [rbp + 480], rbx
	lea rdi, [rbp + 488]
	mov qword ptr [rbp + 488], r14
	mov qword ptr [rbp + 496], r15
	mov qword ptr [rbp + 504], r12
	mov qword ptr [rbp + 512], r13
	mov qword ptr [rbp + 520], rsi
	mov qword ptr [rbp + 528], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 209
		let mut acc = init;
	mov qword ptr [rbp + 304], rbx
	movdqu xmm0, xmmword ptr [rbp + 464]
	movdqa xmmword ptr [rbp + 288], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 215
		acc = f(acc, unsafe { & $( $mut_ )? *self.ptr.add(i).as_ptr() });
	mov qword ptr [rbp + 400], rbx
	movdqa xmmword ptr [rbp + 384], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 41
		let combined = Self::reduce(|a, b| a | b);

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]

	mov r8, rdi
	call <ecs::layout::access::LayoutAccess as core::ops::bit::BitOr>::bitor

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 215
		acc = f(acc, unsafe { & $( $mut_ )? *self.ptr.add(i).as_ptr() });
	lea r8, [rbp + 512]

	mov rax, qword ptr [rbp + 304]
	mov qword ptr [rbp + 400], rax
	movdqa xmm0, xmmword ptr [rbp + 288]
	movdqa xmmword ptr [rbp + 384], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 41
		let combined = Self::reduce(|a, b| a | b);

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]

	call <ecs::layout::access::LayoutAccess as core::ops::bit::BitOr>::bitor

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 224
		acc
	movdqa xmm0, xmmword ptr [rbp + 288]
	movdqa xmmword ptr [rbp + 96], xmm0
	mov rax, qword ptr [rbp + 304]
	mov qword ptr [rbp + 112], rax
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 42
		let refmut_collisions = combined.shared() & combined.unique() != Mask::zero();

	lea rcx, [rbp + 96]

	call ecs::layout::access::LayoutAccess::shared

	mov rsi, rax
	lea rcx, [rbp + 96]
	call ecs::layout::access::LayoutAccess::unique

	mov rcx, rsi
	mov rdx, rax
	call <ecs::mask::Mask as core::ops::bit::BitAnd>::bitand
	mov qword ptr [rbp + 616], rax

	call ecs::mask::Mask::zero
	mov qword ptr [rbp + 608], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 225
		arch_search: mask::<T>(),

	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 226
		validation_shared: Mask::zero(),

	mov r14, rax
	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 227
		validation_unique: mask::<T>(),

	mov rsi, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov rbx, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 168
		validation_shared: mask::<T>(),

	mov r15, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 169
		validation_unique: Mask::zero(),

	mov r12, rax
	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov r13, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),
	call ecs::registry::mask
	mov qword ptr [rbp + 600], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 168
		validation_shared: mask::<T>(),

	call ecs::registry::mask
	mov qword ptr [rbp + 584], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 169
		validation_unique: Mask::zero(),

	call ecs::mask::Mask::zero
	mov qword ptr [rbp + 592], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 209
		let mut acc = init;
	mov qword ptr [rbp + 224], r14
	mov qword ptr [rbp + 232], rsi
	mov qword ptr [rbp + 240], rbx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 215
		acc = f(acc, unsafe { & $( $mut_ )? *self.ptr.add(i).as_ptr() });
	mov qword ptr [rbp + 480], rbx
	mov rax, qword ptr [rbp + 224]
	mov qword ptr [rbp + 464], rax
	mov rax, qword ptr [rbp + 232]
	mov qword ptr [rbp + 472], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 124
		layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
	mov qword ptr [rbp + 488], r15
	mov qword ptr [rbp + 496], r12
	mov qword ptr [rbp + 504], r13

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 47
		mut_collisions |= (acc.unique() & b.unique()) == b.unique() && (!b.unique().is_zero());

	lea rcx, [rbp + 464]

	call ecs::layout::access::LayoutAccess::unique

	mov rsi, rax

	mov rcx, rdi
	call ecs::layout::access::LayoutAccess::unique

	mov rcx, rsi
	mov rdx, rax
	call <ecs::mask::Mask as core::ops::bit::BitAnd>::bitand

	mov rsi, rax
	mov rcx, rdi
	call ecs::layout::access::LayoutAccess::unique

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 21
		#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
	cmp rsi, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 47
		mut_collisions |= (acc.unique() & b.unique()) == b.unique() && (!b.unique().is_zero());
	jne .LBB77_258

	mov rcx, rdi
	call ecs::layout::access::LayoutAccess::unique

	mov qword ptr [rbp + 384], rax

	lea rcx, [rbp + 384]
	call ecs::mask::Mask::is_zero

	mov r14d, eax

	xor r14b, 1
	jmp .LBB77_262

.LBB77_258:
	xor r14d, r14d

.LBB77_262:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 48
		acc | b
	mov rax, qword ptr [rbp + 480]
	mov qword ptr [rbp + 304], rax
	movaps xmm0, xmmword ptr [rbp + 464]
	movaps xmmword ptr [rbp + 288], xmm0
	mov rax, qword ptr [rdi + 16]
	mov qword ptr [rbp + 400], rax
	movdqu xmm0, xmmword ptr [rdi]
	movdqa xmmword ptr [rbp + 384], xmm0

	lea rcx, [rbp + 224]
	lea rdx, [rbp + 288]
	lea r8, [rbp + 384]
	call <ecs::layout::access::LayoutAccess as core::ops::bit::BitOr>::bitor

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 215
		acc = f(acc, unsafe { & $( $mut_ )? *self.ptr.add(i).as_ptr() });
	mov rax, qword ptr [rbp + 240]
	mov qword ptr [rbp + 480], rax
	movdqu xmm0, xmmword ptr [rbp + 224]
	movdqa xmmword ptr [rbp + 464], xmm0

	mov rax, qword ptr [rbp + 600]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 124
		layouts[..].into_iter().cloned().reduce(|a, b| lambda(a, b)).unwrap()
	mov qword ptr [rbp + 488], rax
	mov rax, qword ptr [rbp + 584]
	mov qword ptr [rbp + 496], rax
	mov rax, qword ptr [rbp + 592]
	mov qword ptr [rbp + 504], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 47
		mut_collisions |= (acc.unique() & b.unique()) == b.unique() && (!b.unique().is_zero());

	lea rcx, [rbp + 464]

	call ecs::layout::access::LayoutAccess::unique

	mov rsi, rax
	mov rcx, rdi
	call ecs::layout::access::LayoutAccess::unique

	mov rcx, rsi
	mov rdx, rax
	call <ecs::mask::Mask as core::ops::bit::BitAnd>::bitand

	mov rsi, rax
	mov rcx, rdi
	call ecs::layout::access::LayoutAccess::unique

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 21
		#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
	cmp rsi, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 47
		mut_collisions |= (acc.unique() & b.unique()) == b.unique() && (!b.unique().is_zero());
	jne .LBB77_268

	mov rcx, rdi
	call ecs::layout::access::LayoutAccess::unique

	mov qword ptr [rbp + 384], rax

	lea rcx, [rbp + 384]
	call ecs::mask::Mask::is_zero

	mov esi, eax
	xor sil, 1
	jmp .LBB77_272

.LBB77_268:
	xor esi, esi

.LBB77_272:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\layout.rs : 48
		acc | b
	mov rax, qword ptr [rbp + 480]
	mov qword ptr [rbp + 304], rax
	movaps xmm0, xmmword ptr [rbp + 464]
	movaps xmmword ptr [rbp + 288], xmm0
	mov rax, qword ptr [rdi + 16]
	mov qword ptr [rbp + 400], rax
	movdqu xmm0, xmmword ptr [rdi]
	movdqa xmmword ptr [rbp + 384], xmm0

	lea rcx, [rbp + 224]
	lea rdx, [rbp + 288]
	lea r8, [rbp + 384]
	call <ecs::layout::access::LayoutAccess as core::ops::bit::BitOr>::bitor

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 21
		#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]
	mov rax, qword ptr [rbp + 608]

	cmp qword ptr [rbp + 616], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\scene.rs : 236
		assert!(
	jne .LBB77_328

	or r14b, sil
	jne .LBB77_328

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 26
		let (access, archetypes, _) = super::archetypes_mut::<L, Always>(scene.archetypes_mut());

	lea rcx, [rbp - 72]

	call ecs::scene::Scene::archetypes_mut

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 225
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 26
		let (access, archetypes, _) = super::archetypes_mut::<L, Always>(scene.archetypes_mut());
	mov rsi, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 225
		arch_search: mask::<T>(),
	call ecs::registry::mask
	mov qword ptr [rbp + 616], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 226
		validation_shared: Mask::zero(),

	call ecs::mask::Mask::zero
	mov qword ptr [rbp + 608], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 227
		validation_unique: mask::<T>(),

	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov rbx, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 168
		validation_shared: mask::<T>(),

	mov r15, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 169
		validation_unique: Mask::zero(),

	mov r12, rax
	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov r13, rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 167
		arch_search: mask::<T>(),
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 168
		validation_shared: mask::<T>(),

	mov rdi, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\items.rs : 169
		validation_unique: Mask::zero(),

	mov r14, rax
	call ecs::mask::Mask::zero

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 123
		let layouts = [$($name::access()),+];
	mov rcx, qword ptr [rbp + 616]
	mov qword ptr [rbp + 464], rcx
	mov rcx, qword ptr [rbp + 608]
	mov qword ptr [rbp + 472], rcx
	mov qword ptr [rbp + 480], rbx
	lea r8, [rbp + 488]
	mov qword ptr [rbp + 488], r15
	mov qword ptr [rbp + 496], r12
	mov qword ptr [rbp + 504], r13
	mov qword ptr [rbp + 512], rdi
	mov qword ptr [rbp + 520], r14
	mov qword ptr [rbp + 528], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 209
		let mut acc = init;
	mov qword ptr [rbp + 304], rbx
	movdqu xmm0, xmmword ptr [rbp + 464]
	movdqa xmmword ptr [rbp + 288], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 215
		acc = f(acc, unsafe { & $( $mut_ )? *self.ptr.add(i).as_ptr() });
	mov qword ptr [rbp + 400], rbx
	movdqa xmmword ptr [rbp + 384], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 47
		let mask = L::reduce(|a, b| a | b);

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]

	call <ecs::layout::access::LayoutAccess as core::ops::bit::BitOr>::bitor

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 215
		acc = f(acc, unsafe { & $( $mut_ )? *self.ptr.add(i).as_ptr() });
	lea r8, [rbp + 512]

	mov rax, qword ptr [rbp + 304]
	mov qword ptr [rbp + 400], rax
	movdqa xmm0, xmmword ptr [rbp + 288]
	movdqa xmmword ptr [rbp + 384], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 47
		let mask = L::reduce(|a, b| a | b);

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]

	call <ecs::layout::access::LayoutAccess as core::ops::bit::BitOr>::bitor

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\iter\macros.rs : 224
		acc
	movdqa xmm0, xmmword ptr [rbp + 288]
	movdqa xmmword ptr [rbp + 224], xmm0
	mov rax, qword ptr [rbp + 304]
	mov qword ptr [rbp + 240], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 48
		let cached = F::prepare();

	call <ecs::query::filters::Always as ecs::query::filters::QueryFilter>::prepare

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 921
	mov r14, qword ptr [rsi]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2121
	mov rax, qword ptr [rsi + 8]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	add rax, r14
	inc rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [r14]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb r13d, xmm0

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 31
	not r13d

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	lea r12, [r14 + 16]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1429
	mov r15, qword ptr [rsi + 24]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 51
		.filter_map(move |(&archetype_mask, archetype)| {
	mov rcx, qword ptr [rbp + 240]
	mov qword ptr [rbp + 440], rcx
	movdqa xmm0, xmmword ptr [rbp + 224]
	movdqu xmmword ptr [rbp + 424], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\adapters\filter.rs : 25
		Filter { iter, predicate }
	mov qword ptr [rbp + 384], r14
	mov qword ptr [rbp + 392], r12
	mov qword ptr [rbp + 400], rax
	mov word ptr [rbp + 408], r13w
	mov qword ptr [rbp + 416], r15
	lea rax, [rbp + 192]

	mov qword ptr [rbp + 448], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3045
	test r15, r15
	je .LBB77_301

	lea rsi, [rbp + 424]

	lea rdi, [rbp + 464]
	jmp .LBB77_290

.LBB77_300:
	test r15, r15
	je .LBB77_301

.LBB77_290:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r13w, r13w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	jne .LBB77_293

.LBB77_291:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [r12]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb r13d, xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	add r14, -1152

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	add r12, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	cmp r13w, -1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_291

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 31
	not r13d

	mov qword ptr [rbp + 392], r12
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2835
	mov qword ptr [rbp + 384], r14

.LBB77_293:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	eax, r13d

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	lea rax, [rax + 8*rax]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 4763
	lea rbx, [r14 + 8*rax]
	add rbx, -64

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\iterator.rs : 2303
		accum = f(accum, x)?;
	mov rax, qword ptr [r14 + 8*rax - 72]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 51
		.filter_map(move |(&archetype_mask, archetype)| {
	mov qword ptr [rbp + 464], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 52
		(!archetype.is_empty() && archetype_mask.contains(mask.search())).then_some(archetype)

	mov rcx, rbx
	call ecs::archetype::archetype::Archetype::is_empty

	lea ecx, [r13 - 1]
	and ecx, r13d
	mov r13d, ecx
	dec r15

	test al, al
	jne .LBB77_300

	mov rcx, rsi
	call ecs::layout::access::LayoutAccess::search

	mov rcx, rdi
	mov rdx, rax
	call ecs::mask::Mask::contains

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\bool.rs : 34
		if self { Some(t) } else { None }
	test al, al
	je .LBB77_300

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 54
		.filter(|a| F::evaluate_archetype(cached, a))

	mov rcx, rbx
	call <ecs::query::filters::Always as ecs::query::filters::QueryFilter>::evaluate_archetype

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\adapters\filter_map.rs : 48
		Some(x) => fold(acc, x),
	test al, al
	je .LBB77_300

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 130
	mov word ptr [rbp + 408], r13w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3055
	mov qword ptr [rbp + 416], r15

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1543
		intrinsics::volatile_load(src)
	movzx eax, byte ptr [rip + __rust_no_alloc_shim_is_unstable]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 102
		__rust_alloc(layout.size(), layout.align())
	mov ecx, 32
	mov edx, 8
	call __rust_alloc

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 187
		let ptr = match result {
	test rax, rax
	je .LBB77_303

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov rsi, rax

	mov qword ptr [rax], rbx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\spec_from_iter_nested.rs : 38
		vector
	mov qword ptr [rbp + 288], rax
	mov qword ptr [rbp + 296], 4
	mov qword ptr [rbp + 304], 1

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\spec_from_iter_nested.rs : 43
		<Vec<T> as SpecExtend<T, I>>::spec_extend(&mut vector, iterator);
	movdqu xmm0, xmmword ptr [rbp + 384]
	movdqu xmm1, xmmword ptr [rbp + 400]
	movdqu xmm2, xmmword ptr [rbp + 416]
	movdqu xmm3, xmmword ptr [rbp + 432]
	movdqa xmmword ptr [rbp + 496], xmm2
	mov rax, qword ptr [rbp + 448]
	mov qword ptr [rbp + 528], rax
	movdqa xmmword ptr [rbp + 512], xmm3
	movdqa xmmword ptr [rbp + 480], xmm1
	movdqa xmmword ptr [rbp + 464], xmm0

	mov r13, qword ptr [rbp + 496]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3045
	test r13, r13
	je .LBB77_322

	mov eax, 1
	mov qword ptr [rbp + 616], rax
	lea r14, [rbp + 96]

.LBB77_306:
	mov r15, qword ptr [rbp + 464]
	mov rbx, qword ptr [rbp + 472]
	movzx edi, word ptr [rbp + 488]
	jmp .LBB77_307

.LBB77_318:
	test r13, r13
	je .LBB77_322

.LBB77_307:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test di, di

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	jne .LBB77_310

.LBB77_308:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [rbx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb edi, xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	add r15, -1152

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	add rbx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	cmp di, -1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_308

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 31
	not edi

	mov qword ptr [rbp + 472], rbx
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2835
	mov qword ptr [rbp + 464], r15

.LBB77_310:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	mov eax, edi
	lea edi, [rax - 1]

	and edi, eax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 130
	mov word ptr [rbp + 488], di

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3055
	dec r13
	mov qword ptr [rbp + 496], r13

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 4763
	test r15, r15

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\iterator.rs : 2302
		while let Some(x) = self.next() {
	je .LBB77_322

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\iterator.rs : 2303
		accum = f(accum, x)?;
	rep bsf	eax, eax
	neg rax
	lea rax, [rax + 8*rax]
	lea rcx, [r15 + 8*rax]
	lea r12, [r15 + 8*rax]
	add r12, -64

	mov rax, qword ptr [rcx - 72]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 51
		.filter_map(move |(&archetype_mask, archetype)| {
	mov qword ptr [rbp + 96], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 52
		(!archetype.is_empty() && archetype_mask.contains(mask.search())).then_some(archetype)

	mov rcx, r12

	call ecs::archetype::archetype::Archetype::is_empty

	test al, al
	jne .LBB77_318

	lea rcx, [rbp + 504]
	call ecs::layout::access::LayoutAccess::search

	mov rcx, r14
	mov rdx, rax
	call ecs::mask::Mask::contains

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\bool.rs : 34
		if self { Some(t) } else { None }
	test al, al
	je .LBB77_318

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 54
		.filter(|a| F::evaluate_archetype(cached, a))

	mov rcx, r12
	call <ecs::query::filters::Always as ecs::query::filters::QueryFilter>::evaluate_archetype

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\adapters\filter_map.rs : 48
		Some(x) => fold(acc, x),
	test al, al
	je .LBB77_318

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2796
		while let Some(element) = iterator.next() {
	test r12, r12
	je .LBB77_322

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2798
		if len == self.capacity() {
	mov rdx, qword ptr [rbp + 616]

	cmp rdx, qword ptr [rbp + 296]
	jne .LBB77_321

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 293
		do_reserve_and_handle(self, len, additional);

	mov r8d, 1
	lea rcx, [rbp + 288]
	call alloc::raw_vec::RawVec<T,A>::reserve::do_reserve_and_handle

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rsi, qword ptr [rbp + 288]
	mov rdx, qword ptr [rbp + 616]

.LBB77_321:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov qword ptr [rsi + 8*rdx], r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 2807
		self.set_len(len + 1);
	inc rdx
	mov qword ptr [rbp + 616], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1366
		self.len = new_len;
	mov qword ptr [rbp + 304], rdx

	mov r13, qword ptr [rbp + 496]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3045
	test r13, r13
	jne .LBB77_306

.LBB77_322:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\spec_from_iter_nested.rs : 44
		vector
	lea rcx, [rbp + 296]

	mov rax, qword ptr [rbp + 288]
	movdqu xmm0, xmmword ptr [rcx]
	movdqa xmmword ptr [rbp + 464], xmm0
	jmp .LBB77_323

.LBB77_301:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 421
		Vec { buf: RawVec::NEW, len: 0 }
	pxor xmm0, xmm0
	movdqa xmmword ptr [rbp + 464], xmm0
	mov eax, 8

.LBB77_323:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\filters.rs : 57
		(mask, archetypes, cached)
	mov rcx, qword ptr [rbp + 240]
	mov qword ptr [rbp + 328], rcx
	movaps xmm0, xmmword ptr [rbp + 224]
	movups xmmword ptr [rbp + 312], xmm0
	movdqa xmm0, xmmword ptr [rbp + 464]
	movdqu xmmword ptr [rbp + 296], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 28
		Self {
	mov qword ptr [rbp + 288], rax
	mov qword ptr [rbp + 336], 0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1901
		if self.len == 0 {
	mov rcx, qword ptr [rbp + 304]
	test rcx, rcx
	je .LBB77_324

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 1905
		self.len -= 1;
	lea rdx, [rcx - 1]
	mov qword ptr [rbp + 304], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1179
		crate::intrinsics::read_via_copy(src)
	mov rdi, qword ptr [rax + 8*rcx - 8]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 219
		self.table.get_mut(&mask::<T>())

	call ecs::registry::mask

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 1461
	cmp qword ptr [rdi + 24], 0
	je .LBB77_343

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 1465
	mov r15, qword ptr [rdi]
	mov r12, qword ptr [rdi + 8]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, rax
	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	lea r13, [r15 - 88]
	xor ecx, ecx
	pcmpeqd xmm1, xmm1
	mov rdx, rax

.LBB77_333:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and rdx, r12

	movdqu xmm2, xmmword ptr [r15 + rdx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r8d, xmm3

.LBB77_334:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r8w, r8w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_335

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r10d, [r8 - 1]
	and r10d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r8, rdx

	and r8, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul r9, r8, 88

	mov r8d, r10d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [r13 + r9], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_334
	jmp .LBB77_338

.LBB77_335:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r8d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_343

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add rdx, rcx

	add rdx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add rcx, 16

	jmp .LBB77_333

.LBB77_338:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rcx, qword ptr [r15 + r9 - 80]
	mov rax, qword ptr [r15 + r9 - 72]

	call qword ptr [rax + 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rsi, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();
	mov rcx, rax
	call qword ptr [rdx + 24]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test rsi, rsi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	je .LBB77_342

	movabs rcx, -1581083357955820586
	xor rax, rcx
	movabs rcx, -7394674131754425909
	xor rdx, rcx
	or rdx, rax
	jne .LBB77_342

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rbx, qword ptr [rsi]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 219
		self.table.get_mut(&mask::<T>())

	call ecs::registry::mask

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, rax
	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	xor ecx, ecx
	pcmpeqd xmm1, xmm1
	mov rdx, rax

.LBB77_346:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and rdx, r12

	movdqu xmm2, xmmword ptr [r15 + rdx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r8d, xmm3

.LBB77_347:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r8w, r8w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_348

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r10d, [r8 - 1]
	and r10d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r8, rdx

	and r8, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul r9, r8, 88

	mov r8d, r10d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [r13 + r9], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_347
	jmp .LBB77_351

.LBB77_348:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r8d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_356

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add rdx, rcx

	add rdx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add rcx, 16

	jmp .LBB77_346

.LBB77_351:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rcx, qword ptr [r15 + r9 - 80]
	mov rax, qword ptr [r15 + r9 - 72]

	call qword ptr [rax + 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rsi, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();
	mov rcx, rax
	call qword ptr [rdx + 24]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test rsi, rsi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	je .LBB77_355

	movabs rcx, -5896397890751364543
	xor rax, rcx
	movabs rcx, 4724347646448115692
	xor rdx, rcx
	or rdx, rax
	jne .LBB77_355

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov r14, qword ptr [rsi]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 219
		self.table.get_mut(&mask::<T>())

	call ecs::registry::mask

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, rax
	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	xor ecx, ecx
	pcmpeqd xmm1, xmm1
	mov rdx, rax

.LBB77_359:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and rdx, r12

	movdqu xmm2, xmmword ptr [r15 + rdx]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm3, xmm0
	pcmpeqb xmm3, xmm2
	pmovmskb r8d, xmm3

.LBB77_360:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r8w, r8w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_361

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r10d, [r8 - 1]
	and r10d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1819
	add r8, rdx

	and r8, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul r9, r8, 88

	mov r8d, r10d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [r13 + r9], rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1821
	jne .LBB77_360
	jmp .LBB77_364

.LBB77_361:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm2, xmm1
	pmovmskb r8d, xmm2

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1826
	jne .LBB77_369

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add rdx, rcx

	add rdx, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add rcx, 16

	jmp .LBB77_359

.LBB77_364:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rcx, qword ptr [r15 + r9 - 80]
	mov rax, qword ptr [r15 + r9 - 72]

	call qword ptr [rax + 32]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	mov rsi, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\any.rs : 261
		let concrete = self.type_id();
	mov rcx, rax
	call qword ptr [rdx + 24]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test rsi, rsi

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\untyped.rs : 98
		let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
	je .LBB77_368

	movabs rcx, 8457430945661996516
	xor rax, rcx
	movabs rcx, 575874667958237832
	xor rdx, rcx
	or rdx, rax
	jne .LBB77_368

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 223
		self.ptr.as_ptr()
	mov rsi, qword ptr [rsi]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 126
		let length = archetype.len();

	mov rcx, rdi
	call ecs::archetype::archetype::Archetype::len

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 129
		archetypes: self.archetypes,
	mov rcx, qword ptr [rbp + 304]
	mov qword ptr [rbp + 544], rcx
	movups xmm0, xmmword ptr [rbp + 288]
	movups xmmword ptr [rbp + 528], xmm0
	lea rcx, [rbp + 336]
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 130
		bitsets: self.bitsets,
	movdqu xmm0, xmmword ptr [rcx]
	movdqu xmmword ptr [rbp + 552], xmm0
	mov rcx, qword ptr [rcx + 16]
	mov qword ptr [rbp + 568], rcx

		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 32
		for (a, b, c) in scene.query_mut::<(&mut Position, &Matrix, &Rotation)>() {
	mov qword ptr [rbp + 464], 1
	mov qword ptr [rbp + 472], 0
	mov qword ptr [rbp + 496], rbx
	mov qword ptr [rbp + 504], r14
	mov qword ptr [rbp + 512], rsi
	mov qword ptr [rbp + 520], rax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 197
		if (self.index >= chunk.length) {
	test rax, rax
	je .LBB77_385

	cmp rax, 12
	jae .LBB77_375

	xor ecx, ecx

.LBB77_380:
	mov r8, rcx
	not r8
	add r8, rax
	mov r9, rax
	and r9, 3
	je .LBB77_381

.LBB77_382:
		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 33
		a.0 = b.0 + c.0; 
	mov r10d, dword ptr [rsi + 4*rcx]
	add r10d, dword ptr [r14 + 4*rcx]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 231
		self.index += 1;
	lea rdx, [rcx + 1]

		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 33
		a.0 = b.0 + c.0; 
	mov dword ptr [rbx + 4*rcx], r10d

	mov rcx, rdx

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 197
		if (self.index >= chunk.length) {
	dec r9
	jne .LBB77_382

	cmp r8, 3
	jae .LBB77_384
	jmp .LBB77_385

.LBB77_375:
	mov rdx, rbx
	sub rdx, r14
	xor ecx, ecx
	cmp rdx, 32
	jb .LBB77_380

	mov rdx, rbx
	sub rdx, rsi
	cmp rdx, 32
	jb .LBB77_380

	mov rcx, rax
	and rcx, -8
	xor edx, edx

.LBB77_378:
		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 33
		a.0 = b.0 + c.0; 
	movdqu xmm0, xmmword ptr [r14 + 4*rdx]
	movdqu xmm1, xmmword ptr [r14 + 4*rdx + 16]
	movdqu xmm2, xmmword ptr [rsi + 4*rdx]
	paddd xmm2, xmm0
	movdqu xmm0, xmmword ptr [rsi + 4*rdx + 16]
	paddd xmm0, xmm1
	movdqu xmmword ptr [rbx + 4*rdx], xmm2
	movdqu xmmword ptr [rbx + 4*rdx + 16], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 231
		self.index += 1;
	add rdx, 8
	cmp rcx, rdx
	jne .LBB77_378

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 197
		if (self.index >= chunk.length) {
	cmp rax, rcx
	jne .LBB77_380
	jmp .LBB77_385

.LBB77_381:
	mov rdx, rcx
	cmp r8, 3
	jb .LBB77_385

.LBB77_384:
		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 33
		a.0 = b.0 + c.0; 
	mov ecx, dword ptr [rsi + 4*rdx]
	add ecx, dword ptr [r14 + 4*rdx]
	mov dword ptr [rbx + 4*rdx], ecx

	mov ecx, dword ptr [rsi + 4*rdx + 4]
	add ecx, dword ptr [r14 + 4*rdx + 4]
	mov dword ptr [rbx + 4*rdx + 4], ecx

	mov ecx, dword ptr [rsi + 4*rdx + 8]
	add ecx, dword ptr [r14 + 4*rdx + 8]
	mov dword ptr [rbx + 4*rdx + 8], ecx

	mov ecx, dword ptr [rsi + 4*rdx + 12]
	add ecx, dword ptr [r14 + 4*rdx + 12]
	mov dword ptr [rbx + 4*rdx + 12], ecx

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 231
		self.index += 1;
	lea rcx, [rdx + 4]
	mov rdx, rcx

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 197
		if (self.index >= chunk.length) {
	cmp rax, rcx
	jne .LBB77_384

.LBB77_385:
		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 32
		for (a, b, c) in scene.query_mut::<(&mut Position, &Matrix, &Rotation)>() {
	mov qword ptr [rbp + 576], rax

	lea rcx, [rbp + 464]

	call core::ptr::drop_in_place<ecs::query::query_mut::QueryMutIter<(&mut cflake_engine::Position,&cflake_engine::Matrix,&cflake_engine::Rotation)>>

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	mov rax, qword ptr [rbp - 64]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 240
		if T::IS_ZST || self.cap == 0 {
	test rax, rax
	je .LBB77_388

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	mov rcx, qword ptr [rbp - 72]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 1267
		uint_impl! {
	shl rax, 3

	lea rdx, [rax + 2*rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 121
		unsafe { __rust_dealloc(ptr, layout.size(), layout.align()) }
	mov r8d, 8
	call __rust_dealloc

.LBB77_388:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {

	lea rcx, [rbp - 40]
	call core::ptr::drop_in_place<std::collections::hash::map::HashMap<ecs::mask::Mask,ecs::archetype::archetype::Archetype,core::hash::BuildHasherDefault<nohash_hasher::NoHashHasher<ecs::mask::Mask>>>>

	lea rcx, [rbp - 8]

	call core::ptr::drop_in_place<hashbrown::raw::RawTable<(ecs::mask::Mask,alloc::boxed::Box<dyn ecs::vec::UntypedVec>)>>

	lea rcx, [rbp + 24]
	call core::ptr::drop_in_place<ahash::hash_map::AHashMap<&str,(alloc::boxed::Box<dyn ecs::layout::bundle::PrefabBundle>,ecs::mask::Mask)>>
	movaps xmm6, xmmword ptr [rbp + 640]

		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 35
		}
	add rsp, 792
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

.LBB77_17:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1084
	mov qword ptr [rbp + 184], r14
	mov qword ptr [rbp + 176], rbx

	cmp qword ptr [rbp - 24], 0
	jne .LBB77_19

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1086

	lea rcx, [rbp - 40]
	call hashbrown::raw::RawTable<T,A>::reserve_rehash

.LBB77_19:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 16
		let masks = [$(mask::<$name>()),+];

	call ecs::registry::mask

	mov rsi, rax
	call ecs::registry::mask

	mov r14, rax
	call ecs::registry::mask

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 26
		let mask = B::reduce(|a, b| a | b);

	mov rbx, rax

	mov rcx, rsi
	mov rdx, r14
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov rcx, rax
	mov rdx, rbx
	call <ecs::mask::Mask as core::ops::bit::BitOr>::bitor

	mov qword ptr [rbp + 168], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\sync\atomic.rs : 3153
		Relaxed => intrinsics::atomic_load_relaxed(dst),
	mov rax, qword ptr [rip + __imp__ZN3log20MAX_LOG_LEVEL_FILTER17he3bdb6ed9dc9e174E]

	mov rax, qword ptr [rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\cmp.rs : 1411
		ord_impl! { char usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }
	xor ecx, ecx
	cmp rax, 4
	setne cl

	mov eax, 255
	cmovbe eax, ecx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\cmp.rs : 1085
		matches!(self.partial_cmp(other), Some(Less | Equal))
	cmp al, -1
	je .LBB77_26

	movzx eax, al
	test eax, eax
	jne .LBB77_27

.LBB77_26:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 28
		log::debug!("Creating archetype from bundle of mask {:?}", mask);
	lea rax, [rbp + 168]
	mov qword ptr [rbp + 384], rax
	lea rax, [rip + <ecs::mask::Mask as core::fmt::Debug>::fmt]
	mov qword ptr [rbp + 392], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\fmt\mod.rs : 311
		Arguments { pieces, fmt: None, args }
	lea rax, [rip + __unnamed_27]

	mov qword ptr [rbp + 464], rax
	mov qword ptr [rbp + 472], 1
	mov qword ptr [rbp + 496], 0
	lea rax, [rbp + 384]

	mov qword ptr [rbp + 480], rax
	mov qword ptr [rbp + 488], 1

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 28
		log::debug!("Creating archetype from bundle of mask {:?}", mask);

	mov qword ptr [rsp + 32], 0
	lea r8, [rip + __unnamed_26]
	lea rcx, [rbp + 464]
	mov edx, 4
	mov r9d, 28
	call log::__private_api::log

.LBB77_27:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 285
		HashMap { base: base::HashMap::with_hasher(hash_builder) }
	lea rbx, [rip + __unnamed_28]
	mov qword ptr [rbp + 464], rbx
	pxor xmm6, xmm6
	movdqu xmmword ptr [rbp + 472], xmm6
	mov qword ptr [rbp + 488], 0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 72
		map.insert(mask::<$name>(), Box::<Vec::<$name>>::default())

	call ecs::registry::mask

	mov rsi, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 421
		Vec { buf: RawVec::NEW, len: 0 }
	mov qword ptr [rbp + 384], 4

	movdqu xmmword ptr [rbp + 392], xmm6

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1543
		intrinsics::volatile_load(src)
	movzx eax, byte ptr [rip + __rust_no_alloc_shim_is_unstable]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 102
		__rust_alloc(layout.size(), layout.align())
	mov ecx, 24
	mov edx, 8
	call __rust_alloc

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 334
		match Global.allocate(layout) {
	test rax, rax
	je .LBB77_29

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\boxed.rs : 217
		Box::new(x)
	mov rcx, qword ptr [rbp + 400]
	mov qword ptr [rax + 16], rcx
	movdqu xmm0, xmmword ptr [rbp + 384]
	movdqu xmmword ptr [rax], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 1103
		self.base.insert(k, v)

	lea r9, [rip + __unnamed_22]

	lea rcx, [rbp + 464]
	mov rdx, rsi
	mov r8, rax
	call hashbrown::map::HashMap<K,V,S,A>::insert

	mov qword ptr [rbp + 600], rax
	mov qword ptr [rbp + 584], rdx

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 72
		map.insert(mask::<$name>(), Box::<Vec::<$name>>::default())

	call ecs::registry::mask

	mov rsi, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 421
		Vec { buf: RawVec::NEW, len: 0 }
	mov qword ptr [rbp + 384], 4

	pxor xmm0, xmm0
	movdqu xmmword ptr [rbp + 392], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1543
		intrinsics::volatile_load(src)
	movzx eax, byte ptr [rip + __rust_no_alloc_shim_is_unstable]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 102
		__rust_alloc(layout.size(), layout.align())
	mov ecx, 24
	mov edx, 8
	call __rust_alloc

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 334
		match Global.allocate(layout) {
	test rax, rax
	je .LBB77_39

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\boxed.rs : 217
		Box::new(x)
	mov rcx, qword ptr [rbp + 400]
	mov qword ptr [rax + 16], rcx
	movdqu xmm0, xmmword ptr [rbp + 384]
	movdqu xmmword ptr [rax], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 1103
		self.base.insert(k, v)

	lea r9, [rip + __unnamed_21]

	lea rcx, [rbp + 464]
	mov rdx, rsi
	mov r8, rax
	call hashbrown::map::HashMap<K,V,S,A>::insert

	mov qword ptr [rbp + 592], rax
	mov qword ptr [rbp + 376], rdx

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 72
		map.insert(mask::<$name>(), Box::<Vec::<$name>>::default())

	call ecs::registry::mask

	mov rsi, rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\mod.rs : 421
		Vec { buf: RawVec::NEW, len: 0 }
	mov qword ptr [rbp + 384], 4

	pxor xmm0, xmm0
	movdqu xmmword ptr [rbp + 392], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1543
		intrinsics::volatile_load(src)
	movzx eax, byte ptr [rip + __rust_no_alloc_shim_is_unstable]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 102
		__rust_alloc(layout.size(), layout.align())
	mov ecx, 24
	mov edx, 8
	call __rust_alloc

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 334
		match Global.allocate(layout) {
	test rax, rax
	je .LBB77_44

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\boxed.rs : 217
		Box::new(x)
	mov rcx, qword ptr [rbp + 400]
	mov qword ptr [rax + 16], rcx
	movdqu xmm0, xmmword ptr [rbp + 384]
	movdqu xmmword ptr [rax], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 1103
		self.base.insert(k, v)

	lea r9, [rip + __unnamed_20]

	lea rcx, [rbp + 464]
	mov rdx, rsi
	mov r8, rax
	call hashbrown::map::HashMap<K,V,S,A>::insert

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	mov rsi, rax
	mov r14, rdx

	mov rcx, qword ptr [rbp + 600]

	test rcx, rcx
	mov rax, qword ptr [rbp + 584]
	mov qword ptr [rbp + 616], rsi
	mov qword ptr [rbp + 608], rdx
	je .LBB77_51

	call qword ptr [rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\mem\mod.rs : 394
		unsafe { intrinsics::size_of_val(val) }
	mov rax, qword ptr [rbp + 584]

	mov rdx, qword ptr [rax + 8]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 255
		if layout.size() != 0 {
	test rdx, rdx
	mov rcx, qword ptr [rbp + 600]
	mov rsi, qword ptr [rbp + 616]
	mov r14, qword ptr [rbp + 608]
	je .LBB77_51

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 121
		unsafe { __rust_dealloc(ptr, layout.size(), layout.align()) }
	mov r8, qword ptr [rax + 16]

	call __rust_dealloc

.LBB77_51:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	mov rcx, qword ptr [rbp + 592]

	test rcx, rcx
	mov rax, qword ptr [rbp + 376]
	je .LBB77_55

	call qword ptr [rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\mem\mod.rs : 394
		unsafe { intrinsics::size_of_val(val) }
	mov rax, qword ptr [rbp + 376]

	mov rdx, qword ptr [rax + 8]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 255
		if layout.size() != 0 {
	test rdx, rdx
	mov rcx, qword ptr [rbp + 592]
	mov rsi, qword ptr [rbp + 616]
	mov r14, qword ptr [rbp + 608]
	je .LBB77_55

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 121
		unsafe { __rust_dealloc(ptr, layout.size(), layout.align()) }
	mov r8, qword ptr [rax + 16]

	call __rust_dealloc

.LBB77_55:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	test rsi, rsi
	je .LBB77_59

	mov rcx, rsi
	call qword ptr [r14]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\mem\mod.rs : 394
		unsafe { intrinsics::size_of_val(val) }
	mov rax, qword ptr [rbp + 608]

	mov rdx, qword ptr [rax + 8]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 255
		if layout.size() != 0 {
	test rdx, rdx
	mov rcx, qword ptr [rbp + 616]
	je .LBB77_59

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 121
		unsafe { __rust_dealloc(ptr, layout.size(), layout.align()) }
	mov r8, qword ptr [rax + 16]

	call __rust_dealloc

.LBB77_59:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 74
		map
	mov r12, qword ptr [rbp + 464]

	mov r15, qword ptr [rbp + 472]

	mov r8, qword ptr [rbp + 488]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [r12]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1499
	test r15, r15
	je .LBB77_60

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2121
	lea rsi, [r15 + 1]

	mov ecx, 24

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 1267
		uint_impl! {
	mov rax, rsi
	mul rcx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 251
	jo .LBB77_65

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 1267
		uint_impl! {
	add rax, 15

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 251
	and rax, -16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 252
	add r15, 17

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 1267
		uint_impl! {
	add r15, rax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 252
	jb .LBB77_65

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 256
	movabs rcx, 9223372036854775793

	cmp r15, rcx
	jb .LBB77_64

.LBB77_65:
	xor ecx, ecx

.LBB77_66:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	mov rdx, r12
	sub rdx, rax
	jmp .LBB77_67

.LBB77_60:
	mov esi, 1
	xor ecx, ecx

.LBB77_67:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	lea r14, [r12 + 16]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb r13d, xmm0

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 31
	not r13d

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	add rsi, r12

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\adapters\map.rs : 69
		Map { iter, f }
	movzx eax, word ptr [rbp + 468]
	mov word ptr [rbp + 372], ax
	mov eax, dword ptr [rbp + 464]
	mov dword ptr [rbp + 368], eax

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 35
		mask,
	mov rax, qword ptr [rbp + 168]

	mov qword ptr [rbp + 600], rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 285
		HashMap { base: base::HashMap::with_hasher(hash_builder) }
	mov qword ptr [rbp + 192], rbx

	pxor xmm0, xmm0
	movdqu xmmword ptr [rbp + 200], xmm0
	mov qword ptr [rbp + 216], 0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\collect.rs : 282
		self
	mov qword ptr [rbp + 96], rdx
	mov qword ptr [rbp + 104], rcx
	mov qword ptr [rbp + 112], r15
	mov qword ptr [rbp + 120], r12
	mov qword ptr [rbp + 128], r14
	mov qword ptr [rbp + 136], rsi
	mov word ptr [rbp + 144], r13w
	mov eax, dword ptr [rbp + 368]
	mov dword ptr [rbp + 146], eax
	movzx eax, word ptr [rbp + 372]
	mov word ptr [rbp + 150], ax
	mov qword ptr [rbp + 152], r8

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1084
	test r8, r8
	je .LBB77_69

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1086
	mov byte ptr [rbp + 631], 1

	mov qword ptr [rbp + 616], rcx

	lea rcx, [rbp + 192]
	mov qword ptr [rbp + 608], rdx

	mov rdx, r8
	mov rbx, r8

	call hashbrown::raw::RawTable<T,A>::reserve_rehash
	mov rdx, qword ptr [rbp + 608]
	mov rcx, qword ptr [rbp + 616]
	mov r8, rbx

.LBB77_69:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 6455
	mov qword ptr [rbp + 224], rdx
	mov qword ptr [rbp + 232], rcx
	mov qword ptr [rbp + 240], r15
	mov qword ptr [rbp + 248], r12
	mov qword ptr [rbp + 256], r14
	mov qword ptr [rbp + 264], rsi
	mov word ptr [rbp + 272], r13w
	mov eax, dword ptr [rbp + 368]
	mov dword ptr [rbp + 274], eax
	movzx eax, word ptr [rbp + 372]
	mov word ptr [rbp + 278], ax
	mov qword ptr [rbp + 280], r8
	pcmpeqd xmm6, xmm6
	jmp .LBB77_70

.LBB77_86:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 118
	and r9b, 1

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\convert\num.rs : 90
		impl_from_bool! { usize, #[stable(feature = "from_bool", since = "1.28.0")] }
	movzx r8d, r9b

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1953
	sub qword ptr [rbp + 208], r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 1267
		uint_impl! {
	lea r8, [r10 - 16]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2082
	and r8, rdx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2085
	mov byte ptr [rax + r10], cl
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2086
	mov byte ptr [r8 + rax + 16], cl

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1955
	inc qword ptr [rbp + 216]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg r10

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul rcx, r10, 88

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov qword ptr [rax + rcx - 88], r13
	movups xmm0, xmmword ptr [rbp + 288]
	movdqu xmm1, xmmword ptr [rbp + 304]
	movdqu xmm2, xmmword ptr [rbp + 320]
	movdqu xmm3, xmmword ptr [rbp + 336]
	movups xmmword ptr [rax + rcx - 80], xmm0
	movdqu xmmword ptr [rax + rcx - 64], xmm1
	movdqu xmmword ptr [rax + rcx - 48], xmm2
	movdqu xmmword ptr [rax + rcx - 32], xmm3
	movdqu xmm0, xmmword ptr [rbp + 352]
	movdqu xmmword ptr [rax + rcx - 16], xmm0

.LBB77_91:
	mov r13d, esi

	mov r8, qword ptr [rbp + 616]

.LBB77_70:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3045
	test r8, r8

	je .LBB77_93

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r13w, r13w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_72

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea esi, [r13 - 1]
	and esi, r13d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 130
	mov word ptr [rbp + 272], si

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3055
	dec r8
	mov qword ptr [rbp + 280], r8

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 2443
		match self {
	test r12, r12
	jne .LBB77_74
	jmp .LBB77_93

.LBB77_72:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [r14]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb r13d, xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	add r12, -384

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\const_ptr.rs : 921
		unsafe { intrinsics::offset(self, count) }
	add r14, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	cmp r13w, -1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_72

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2835
	mov qword ptr [rbp + 256], r14
	mov qword ptr [rbp + 248], r12

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	mov eax, -2
	sub eax, r13d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 31
	not r13d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	mov esi, r13d
	and esi, eax

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 130
	mov word ptr [rbp + 272], si

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3055
	dec r8
	mov qword ptr [rbp + 280], r8

.LBB77_74:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	eax, r13d

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg rax

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	lea rax, [rax + 2*rax]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3133
	mov rdx, qword ptr [r12 + 8*rax - 16]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\iterator.rs : 2480
		while let Some(x) = self.next() {
	test rdx, rdx
	je .LBB77_93

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 3133
	mov qword ptr [rbp + 616], r8

	lea rcx, [r12 + 8*rax]
	add rcx, -24
	mov r13, qword ptr [r12 + 8*rax - 24]
	mov r8, qword ptr [rcx + 16]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 32
		.map(|(mask, vec)| (mask, UntypedColumn::new(vec)));

	lea rcx, [rbp + 288]
	call ecs::archetype::untyped::UntypedColumn::new

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 6455
	movups xmm0, xmmword ptr [rbp + 352]
	movaps xmmword ptr [rbp + 448], xmm0
	movdqu xmm0, xmmword ptr [rbp + 288]
	movdqu xmm1, xmmword ptr [rbp + 304]
	movdqu xmm2, xmmword ptr [rbp + 320]
	movdqu xmm3, xmmword ptr [rbp + 336]
	movdqa xmmword ptr [rbp + 432], xmm3
	movdqa xmmword ptr [rbp + 416], xmm2
	movdqa xmmword ptr [rbp + 400], xmm1
	movdqa xmmword ptr [rbp + 384], xmm0

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1084
	cmp qword ptr [rbp + 208], 0
	jne .LBB77_78

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1086

	mov edx, 1
	lea rcx, [rbp + 192]
	call hashbrown::raw::RawTable<T,A>::reserve_rehash

.LBB77_78:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1264
	mov rax, qword ptr [rbp + 192]
	mov rdx, qword ptr [rbp + 200]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 144
	mov rcx, r13
	shr rcx, 57
	movd xmm0, ecx
	punpcklbw xmm0, xmm0
	pshuflw xmm0, xmm0, 0
	pshufd xmm0, xmm0, 0
	xor r9d, r9d
	mov r10, r13
	xor r11d, r11d

.LBB77_79:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	and r10, rdx

	movdqu xmm1, xmmword ptr [rax + r10]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	movdqa xmm2, xmm0
	pcmpeqb xmm2, xmm1
	pmovmskb r15d, xmm2

.LBB77_80:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	test r15w, r15w

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 129
	je .LBB77_81

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 38
	lea r8d, [r15 - 1]
	and r8d, r15d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	ebx, r15d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1696
	add rbx, r10

	and rbx, rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg rbx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	imul rbx, rbx, 88

	mov r15d, r8d

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\mask.rs : 22
		pub struct Mask(RawBitMask);
	cmp qword ptr [rax + rbx - 88], r13

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1698
	jne .LBB77_80
	jmp .LBB77_89

.LBB77_81:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 599
		matches!(*self, Some(_))
	cmp r11, 1
	mov r11d, 1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1705
	je .LBB77_82

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb r8d, xmm1

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\nonzero.rs : 165
		nonzero_integers! {
	xor r11d, r11d
	test r8d, r8d
	setne r11b

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1669
	add r8, r10

	and r8, rdx

	mov qword ptr [rbp + 608], r8

.LBB77_82:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pcmpeqb xmm1, xmm6
	pmovmskb r8d, xmm1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 44
	test r8d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1711
	jne .LBB77_84

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 172
	add r10, r9

	add r10, 16

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 171
	add r9, 16

	jmp .LBB77_79

.LBB77_89:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1179
		crate::intrinsics::read_via_copy(src)
	add rax, rbx

	movups xmm0, xmmword ptr [rax - 16]
	movaps xmmword ptr [rbp + 528], xmm0
	movups xmm0, xmmword ptr [rax - 80]
	movups xmm1, xmmword ptr [rax - 64]
	movups xmm2, xmmword ptr [rax - 48]
	movups xmm3, xmmword ptr [rax - 32]
	movaps xmmword ptr [rbp + 512], xmm3
	movaps xmmword ptr [rbp + 496], xmm2
	movaps xmmword ptr [rbp + 480], xmm1
	movaps xmmword ptr [rbp + 464], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	movups xmm0, xmmword ptr [rbp + 352]
	movups xmmword ptr [rax - 16], xmm0
	movdqu xmm0, xmmword ptr [rbp + 288]
	movdqu xmm1, xmmword ptr [rbp + 304]
	movdqu xmm2, xmmword ptr [rbp + 320]
	movdqu xmm3, xmmword ptr [rbp + 336]
	movdqu xmmword ptr [rax - 32], xmm3
	movdqu xmmword ptr [rax - 48], xmm2
	movdqu xmmword ptr [rax - 64], xmm1
	movdqu xmmword ptr [rax - 80], xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	cmp qword ptr [rbp + 464], 0

	je .LBB77_91

	lea rcx, [rbp + 464]
	call core::ptr::drop_in_place<ecs::archetype::untyped::UntypedColumn>

	jmp .LBB77_91

.LBB77_84:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2132
	mov r10, qword ptr [rbp + 608]

	movzx r9d, byte ptr [rax + r10]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 105
	test r9b, r9b

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1639
	js .LBB77_86

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb r8d, xmm0

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	r10d, r8d

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1284
	movzx r9d, byte ptr [rax + r10]
	jmp .LBB77_86

.LBB77_93:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\iterator.rs : 2484
		}
	mov byte ptr [rbp + 631], 0

	lea rcx, [rbp + 224]
	call core::ptr::drop_in_place<std::collections::hash::map::IntoIter<ecs::mask::Mask,alloc::boxed::Box<dyn ecs::vec::UntypedVec>>>

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 3022
		map
	movups xmm0, xmmword ptr [rbp + 192]
	movdqu xmm1, xmmword ptr [rbp + 208]
	movaps xmmword ptr [rbp + 464], xmm0
	movdqa xmmword ptr [rbp + 480], xmm1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1210
	mov rax, qword ptr [rbp - 40]
	mov rcx, qword ptr [rbp - 32]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1931
	and rdi, rcx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	movdqu xmm0, xmmword ptr [rax + rdi]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb edx, xmm0

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\nonzero.rs : 165
		nonzero_integers! {
	test edx, edx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1778
	je .LBB77_100

	mov r10, qword ptr [rbp + 176]
	mov r11, qword ptr [rbp + 184]
	mov rsi, qword ptr [rbp + 600]
	jmp .LBB77_102

.LBB77_100:
	mov r8d, 16
	mov r10, qword ptr [rbp + 176]
	mov r11, qword ptr [rbp + 184]
	mov rsi, qword ptr [rbp + 600]

.LBB77_101:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 173
	add rdi, r8

	and rdi, rcx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\intrinsics.rs : 2680
		copy_nonoverlapping(src, dst, count)
	movdqu xmm0, xmmword ptr [rax + rdi]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb edx, xmm0

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1778
	add r8, 16

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\nonzero.rs : 165
		nonzero_integers! {
	test edx, edx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1778
	je .LBB77_101

.LBB77_102:
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	edx, edx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1669
	add rdx, rdi

	and rdx, rcx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2132
	movzx r8d, byte ptr [rax + rdx]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 105
	test r8b, r8b

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1639
	js .LBB77_104

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1188
		*mem_addr
	movdqa xmm0, xmmword ptr [rax]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\..\..\stdarch\crates\core_arch\src\x86\sse2.rs : 1389
		simd_bitmask::<_, u16>(m) as u32 as i32
	pmovmskb edx, xmm0

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\bitmask.rs : 50
	rep bsf	edx, edx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1731
	movzx r8d, byte ptr [rax + rdx]

.LBB77_104:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 1267
		uint_impl! {
	lea r9, [rdx - 16]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2082
	and r9, rcx

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2085
	mov byte ptr [rax + rdx], r11b
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 2086
	mov byte ptr [r9 + rax + 16], r11b

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\num\mod.rs : 456
		int_impl! {
	neg rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mut_ptr.rs : 479
		unsafe { intrinsics::offset(self, count) }
	lea rcx, [rdx + 8*rdx]
	lea r9, [rax + 8*rcx]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 118
	and r8b, 1

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1215
	movzx edx, r8b

	sub qword ptr [rbp - 24], rdx

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 1377
		intrinsics::write_via_move(dst, src)
	mov qword ptr [rax + 8*rcx - 72], r10
	movdqa xmm0, xmmword ptr [rbp + 464]
	movdqa xmm1, xmmword ptr [rbp + 480]
	movdqu xmmword ptr [rax + 8*rcx - 64], xmm0
	movdqu xmmword ptr [rax + 8*rcx - 48], xmm1
	mov qword ptr [rax + 8*rcx - 32], 4
	pxor xmm0, xmm0
	movdqu xmmword ptr [rax + 8*rcx - 24], xmm0
	mov qword ptr [rax + 8*rcx - 8], rsi

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\raw\mod.rs : 1218
	inc qword ptr [rbp - 16]
	jmp .LBB77_105

.LBB77_64:
	mov ecx, 16
	jmp .LBB77_66

.LBB77_210:
		// C:\Users\jribi\.cargo\registry\src\index.crates.io-6f17d22bba15001f\slotmap-1.0.6\src\basic.rs : 356
		panic!("SlotMap number of elements overflow");

	call std::panicking::begin_panic

	jmp .LBB77_31

.LBB77_202:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 935
		None => panic("called `Option::unwrap()` on a `None` value"),

	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_29]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_343:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_30]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_356:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_31]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_369:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_31]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_30:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\fmt\mod.rs : 301
		Arguments { pieces, fmt: None, args: &[] }
	lea rax, [rip + __unnamed_32]

	mov qword ptr [rbp + 464], rax
	mov qword ptr [rbp + 472], 1
	lea rax, [rip + __unnamed_33]

	mov qword ptr [rbp + 480], rax
	pxor xmm0, xmm0
	movdqu xmmword ptr [rbp + 488], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\scene.rs : 69
		assert!(

	lea rdx, [rip + __unnamed_34]
	lea rcx, [rbp + 464]
	call core::panicking::panic_fmt

	jmp .LBB77_31

.LBB77_134:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 79
		assert_eq!(self.mask(), B::reduce(|a, b| a | b));
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_201:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\fmt\mod.rs : 301
		Arguments { pieces, fmt: None, args: &[] }
	lea rax, [rip + __unnamed_32]

	mov qword ptr [rbp + 464], rax
	mov qword ptr [rbp + 472], 1
	lea rax, [rip + __unnamed_33]

	mov qword ptr [rbp + 480], rax
	pxor xmm0, xmm0
	movdqu xmmword ptr [rbp + 488], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\archetype\archetype.rs : 80
		assert!(

	lea rdx, [rip + __unnamed_35]
	lea rcx, [rbp + 464]
	call core::panicking::panic_fmt

	jmp .LBB77_31

.LBB77_141:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 25
		assert!(Self::is_valid());

	lea rcx, [rip + __unnamed_36]
	lea r8, [rip + __unnamed_13]
	mov edx, 34
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_140:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 935
		None => panic("called `Option::unwrap()` on a `None` value"),

	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_37]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_153:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_37]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_165:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_37]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_196:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 62
		assert_eq!(column~N.0.len(), column~N.1.len());
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_197:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 63
		assert_eq!(column~N.0.len(), column~N.2.len());
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_198:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 62
		assert_eq!(column~N.0.len(), column~N.1.len());
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_199:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 63
		assert_eq!(column~N.0.len(), column~N.2.len());
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_200:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 62
		assert_eq!(column~N.0.len(), column~N.1.len());
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_184:
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 63
		assert_eq!(column~N.0.len(), column~N.2.len());
	mov qword ptr [rbp + 464], 0

	lea rcx, [rbp + 288]
	lea rdx, [rbp + 384]
	lea r8, [rbp + 464]
	call core::panicking::assert_failed

	jmp .LBB77_31

.LBB77_227:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\slice\index.rs : 497
		slice_start_index_len_fail(self.start, slice.len());

	lea r8, [rip + __unnamed_38]
	call core::slice::index::slice_start_index_len_fail

	jmp .LBB77_31

.LBB77_328:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\fmt\mod.rs : 301
		Arguments { pieces, fmt: None, args: &[] }
	lea rax, [rip + __unnamed_39]

	mov qword ptr [rbp + 464], rax
	mov qword ptr [rbp + 472], 1
	lea rax, [rip + __unnamed_33]

	mov qword ptr [rbp + 480], rax
	pxor xmm0, xmm0
	movdqu xmmword ptr [rbp + 488], xmm0

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\scene.rs : 236
		assert!(

	lea rdx, [rip + __unnamed_40]
	lea rcx, [rbp + 464]
	call core::panicking::panic_fmt

	jmp .LBB77_31

.LBB77_324:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\option.rs : 935
		None => panic("called `Option::unwrap()` on a `None` value"),

	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_41]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_342:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_37]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_355:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_37]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_368:
	lea rcx, [rip + __unnamed_4]
	lea r8, [rip + __unnamed_37]
	mov edx, 43
	call core::panicking::panic

	jmp .LBB77_31

.LBB77_303:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\raw_vec.rs : 189
		Err(_) => handle_alloc_error(layout),

	mov ecx, 8
	mov edx, 32
	call alloc::alloc::handle_alloc_error

	jmp .LBB77_31

.LBB77_29:
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\alloc.rs : 336
		Err(_) => handle_alloc_error(layout),

	mov ecx, 8
	mov edx, 24
	call alloc::alloc::handle_alloc_error

	jmp .LBB77_31

.LBB77_39:
	mov ecx, 8
	mov edx, 24
	call alloc::alloc::handle_alloc_error

	jmp .LBB77_31

.LBB77_44:
	mov ecx, 8
	mov edx, 24
	call alloc::alloc::handle_alloc_error

.LBB77_31:
	ud2

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 464]
		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 75
		}
	call core::ptr::drop_in_place<std::collections::hash::map::HashMap<ecs::mask::Mask,alloc::boxed::Box<dyn ecs::vec::UntypedVec>,core::hash::BuildHasherDefault<nohash_hasher::NoHashHasher<ecs::mask::Mask>>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 384]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\boxed.rs : 218
		}
	call core::ptr::drop_in_place<alloc::vec::Vec<cflake_engine::Matrix>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 384]

	call core::ptr::drop_in_place<alloc::vec::Vec<cflake_engine::Matrix>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 384]

	call core::ptr::drop_in_place<alloc::vec::Vec<cflake_engine::Matrix>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 384]

		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 1762
	call core::ptr::drop_in_place<ecs::archetype::untyped::UntypedColumn>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13

	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
		// C:\Users\runneradmin\.cargo\registry\src\index.crates.io-6f17d22bba15001f\hashbrown-0.14.0\src\map.rs : 6458
	cmp byte ptr [rbp + 631], 0
	je .LBB77_97

	lea rcx, [rbp + 96]
	call core::ptr::drop_in_place<core::iter::adapters::map::Map<std::collections::hash::map::IntoIter<ecs::mask::Mask,alloc::boxed::Box<dyn ecs::vec::UntypedVec>>,ecs::archetype::archetype::Archetype::from_bundle<(cflake_engine::Position,cflake_engine::Rotation,cflake_engine::Matrix)>::{{closure}}>>

.LBB77_97:
	movaps xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movaps xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 192]
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\std\src\collections\hash\map.rs : 3023
		}
	call core::ptr::drop_in_place<std::collections::hash::map::HashMap<ecs::mask::Mask,ecs::archetype::untyped::UntypedColumn,core::hash::BuildHasherDefault<nohash_hasher::NoHashHasher<ecs::mask::Mask>>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]
	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp - 72]
		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 35
		}
	call core::ptr::drop_in_place<ecs::scene::Scene>
	movaps xmm6, xmmword ptr [rsp + 48]
		// C:\Users\jribi\Projects\Rust\cflake-engine\src\main.rs : 14
		fn main() {
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp

	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\alloc\src\vec\spec_from_iter_nested.rs : 45
		}
	mov rcx, qword ptr [rbp + 288]
	mov rdx, qword ptr [rbp + 296]
	call core::ptr::drop_in_place<alloc::vec::Vec<&mut ecs::archetype::archetype::Archetype>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]
	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 288]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\query\query_mut.rs : 139
		}
	call core::ptr::drop_in_place<ecs::query::query_mut::QueryMut<(&mut cflake_engine::Position,&cflake_engine::Matrix,&cflake_engine::Rotation)>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp

	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]
	movdqa xmmword ptr [rsp + 48], xmm6
	xor ecx, ecx
	call core::ptr::drop_in_place<core::option::Option<utils::bitset::bitset::BitSet<u64>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi

	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]
	movdqa xmmword ptr [rsp + 48], xmm6
	mov rcx, qword ptr [rbp + 600]
	mov rdx, qword ptr [rbp + 584]

		// C:\Users\jribi\Projects\Rust\cflake-engine\crates\ecs\src\layout\macros.rs : 73
		),+);
	call core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<dyn ecs::vec::UntypedVec>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	mov rcx, qword ptr [rbp + 592]
	mov rdx, qword ptr [rbp + 376]
	call core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<dyn ecs::vec::UntypedVec>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	mov rcx, qword ptr [rbp + 600]
	mov rdx, qword ptr [rbp + 584]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
	mov rcx, qword ptr [rbp + 592]
	mov rdx, qword ptr [rbp + 376]

	call core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<dyn ecs::vec::UntypedVec>>>
	movaps xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	mov rcx, qword ptr [rbp + 592]
	mov rdx, qword ptr [rbp + 376]

	call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
	movaps xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movaps xmmword ptr [rsp + 48], xmm6
	mov rcx, qword ptr [rbp + 616]
	mov rdx, qword ptr [rbp + 608]
	call core::ptr::drop_in_place<core::option::Option<alloc::boxed::Box<dyn ecs::vec::UntypedVec>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	mov rcx, qword ptr [rbp + 616]
	mov rdx, qword ptr [rbp + 608]

	call <alloc::boxed::Box<T,A> as core::ops::drop::Drop>::drop
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
	mov byte ptr [rbp + 631], 0
	lea rcx, [rbp + 224]

		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\iter\traits\iterator.rs : 2484
		}
	call core::ptr::drop_in_place<std::collections::hash::map::IntoIter<ecs::mask::Mask,alloc::boxed::Box<dyn ecs::vec::UntypedVec>>>
	mov byte ptr [rbp + 631], 0
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp

	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]

	movdqa xmmword ptr [rsp + 48], xmm6
		// /rustc/d5c2e9c342b358556da91d61ed4133f6f50fc0c3\library\core\src\ptr\mod.rs : 497
		pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
	lea rcx, [rbp - 8]
	call core::ptr::drop_in_place<std::collections::hash::map::HashMap<ecs::mask::Mask,alloc::boxed::Box<dyn ecs::vec::UntypedVec>,core::hash::BuildHasherDefault<nohash_hasher::NoHashHasher<ecs::mask::Mask>>>>
	movdqa xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

	mov qword ptr [rsp + 16], rdx
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rsi
	push rdi
	push rbx
	sub rsp, 72
	lea rbp, [rdx + 128]
	movdqa xmmword ptr [rsp + 48], xmm6
	lea rcx, [rbp + 24]
	call core::ptr::drop_in_place<ahash::hash_map::AHashMap<&str,(alloc::boxed::Box<dyn ecs::layout::bundle::PrefabBundle>,ecs::mask::Mask)>>
	movaps xmm6, xmmword ptr [rsp + 48]
	add rsp, 72
	pop rbx
	pop rdi
	pop rsi
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret

