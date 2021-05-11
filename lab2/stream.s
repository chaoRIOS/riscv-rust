
stream.o:     file format elf64-littleriscv


Disassembly of section .text:

00000000000100b0 <register_fini>:
   100b0:	00000793          	li	a5,0
   100b4:	c789                	beqz	a5,100be <register_fini+0xe>
   100b6:	6541                	lui	a0,0x10
   100b8:	3ee50513          	addi	a0,a0,1006 # 103ee <__libc_fini_array>
   100bc:	a69d                	j	10422 <atexit>
   100be:	8082                	ret

00000000000100c0 <_start>:
   100c0:	00002197          	auipc	gp,0x2
   100c4:	37018193          	addi	gp,gp,880 # 12430 <__global_pointer$>
   100c8:	81818513          	addi	a0,gp,-2024 # 11c48 <completed.1>
   100cc:	01802617          	auipc	a2,0x1802
   100d0:	bb460613          	addi	a2,a2,-1100 # 1811c80 <__BSS_END__>
   100d4:	8e09                	sub	a2,a2,a0
   100d6:	4581                	li	a1,0
   100d8:	1a8000ef          	jal	ra,10280 <memset>
   100dc:	00000517          	auipc	a0,0x0
   100e0:	34650513          	addi	a0,a0,838 # 10422 <atexit>
   100e4:	c519                	beqz	a0,100f2 <_start+0x32>
   100e6:	00000517          	auipc	a0,0x0
   100ea:	30850513          	addi	a0,a0,776 # 103ee <__libc_fini_array>
   100ee:	334000ef          	jal	ra,10422 <atexit>
   100f2:	124000ef          	jal	ra,10216 <__libc_init_array>
   100f6:	4502                	lw	a0,0(sp)
   100f8:	002c                	addi	a1,sp,8
   100fa:	4601                	li	a2,0
   100fc:	054000ef          	jal	ra,10150 <main>
   10100:	a8dd                	j	101f6 <exit>

0000000000010102 <__do_global_dtors_aux>:
   10102:	8181c703          	lbu	a4,-2024(gp) # 11c48 <completed.1>
   10106:	e715                	bnez	a4,10132 <__do_global_dtors_aux+0x30>
   10108:	1141                	addi	sp,sp,-16
   1010a:	e022                	sd	s0,0(sp)
   1010c:	843e                	mv	s0,a5
   1010e:	e406                	sd	ra,8(sp)
   10110:	00000793          	li	a5,0
   10114:	cb81                	beqz	a5,10124 <__do_global_dtors_aux+0x22>
   10116:	6545                	lui	a0,0x11
   10118:	4cc50513          	addi	a0,a0,1228 # 114cc <__FRAME_END__>
   1011c:	00000097          	auipc	ra,0x0
   10120:	000000e7          	jalr	zero # 0 <register_fini-0x100b0>
   10124:	4785                	li	a5,1
   10126:	60a2                	ld	ra,8(sp)
   10128:	80f18c23          	sb	a5,-2024(gp) # 11c48 <completed.1>
   1012c:	6402                	ld	s0,0(sp)
   1012e:	0141                	addi	sp,sp,16
   10130:	8082                	ret
   10132:	8082                	ret

0000000000010134 <frame_dummy>:
   10134:	00000793          	li	a5,0
   10138:	cb99                	beqz	a5,1014e <frame_dummy+0x1a>
   1013a:	65c9                	lui	a1,0x12
   1013c:	6545                	lui	a0,0x11
   1013e:	c5058593          	addi	a1,a1,-944 # 11c50 <object.0>
   10142:	4cc50513          	addi	a0,a0,1228 # 114cc <__FRAME_END__>
   10146:	00000317          	auipc	t1,0x0
   1014a:	00000067          	jr	zero # 0 <register_fini-0x100b0>
   1014e:	8082                	ret

0000000000010150 <main>:
   10150:	1101                	addi	sp,sp,-32
   10152:	ec22                	sd	s0,24(sp)
   10154:	1000                	addi	s0,sp,32
   10156:	fe042623          	sw	zero,-20(s0)
   1015a:	fe042623          	sw	zero,-20(s0)
   1015e:	a8bd                	j	101dc <main+0x8c>
   10160:	fec42703          	lw	a4,-20(s0)
   10164:	67c9                	lui	a5,0x12
   10166:	c8078693          	addi	a3,a5,-896 # 11c80 <a>
   1016a:	fec42783          	lw	a5,-20(s0)
   1016e:	078e                	slli	a5,a5,0x3
   10170:	97b6                	add	a5,a5,a3
   10172:	e398                	sd	a4,0(a5)
   10174:	fec42703          	lw	a4,-20(s0)
   10178:	67c9                	lui	a5,0x12
   1017a:	c8078693          	addi	a3,a5,-896 # 11c80 <a>
   1017e:	fec42783          	lw	a5,-20(s0)
   10182:	078e                	slli	a5,a5,0x3
   10184:	97b6                	add	a5,a5,a3
   10186:	639c                	ld	a5,0(a5)
   10188:	973e                	add	a4,a4,a5
   1018a:	008127b7          	lui	a5,0x812
   1018e:	c8078693          	addi	a3,a5,-896 # 811c80 <b>
   10192:	fec42783          	lw	a5,-20(s0)
   10196:	078e                	slli	a5,a5,0x3
   10198:	97b6                	add	a5,a5,a3
   1019a:	e398                	sd	a4,0(a5)
   1019c:	67c9                	lui	a5,0x12
   1019e:	c8078713          	addi	a4,a5,-896 # 11c80 <a>
   101a2:	fec42783          	lw	a5,-20(s0)
   101a6:	078e                	slli	a5,a5,0x3
   101a8:	97ba                	add	a5,a5,a4
   101aa:	6398                	ld	a4,0(a5)
   101ac:	008127b7          	lui	a5,0x812
   101b0:	c8078693          	addi	a3,a5,-896 # 811c80 <b>
   101b4:	fec42783          	lw	a5,-20(s0)
   101b8:	078e                	slli	a5,a5,0x3
   101ba:	97b6                	add	a5,a5,a3
   101bc:	639c                	ld	a5,0(a5)
   101be:	973e                	add	a4,a4,a5
   101c0:	010127b7          	lui	a5,0x1012
   101c4:	c8078693          	addi	a3,a5,-896 # 1011c80 <c>
   101c8:	fec42783          	lw	a5,-20(s0)
   101cc:	078e                	slli	a5,a5,0x3
   101ce:	97b6                	add	a5,a5,a3
   101d0:	e398                	sd	a4,0(a5)
   101d2:	fec42783          	lw	a5,-20(s0)
   101d6:	2785                	addiw	a5,a5,1
   101d8:	fef42623          	sw	a5,-20(s0)
   101dc:	fec42783          	lw	a5,-20(s0)
   101e0:	0007871b          	sext.w	a4,a5
   101e4:	001007b7          	lui	a5,0x100
   101e8:	f6f74ce3          	blt	a4,a5,10160 <main+0x10>
   101ec:	4781                	li	a5,0
   101ee:	853e                	mv	a0,a5
   101f0:	6462                	ld	s0,24(sp)
   101f2:	6105                	addi	sp,sp,32
   101f4:	8082                	ret

00000000000101f6 <exit>:
   101f6:	1141                	addi	sp,sp,-16
   101f8:	4581                	li	a1,0
   101fa:	e022                	sd	s0,0(sp)
   101fc:	e406                	sd	ra,8(sp)
   101fe:	842a                	mv	s0,a0
   10200:	12a000ef          	jal	ra,1032a <__call_exitprocs>
   10204:	67c9                	lui	a5,0x12
   10206:	c307b503          	ld	a0,-976(a5) # 11c30 <_global_impure_ptr>
   1020a:	6d3c                	ld	a5,88(a0)
   1020c:	c391                	beqz	a5,10210 <exit+0x1a>
   1020e:	9782                	jalr	a5
   10210:	8522                	mv	a0,s0
   10212:	292000ef          	jal	ra,104a4 <_exit>

0000000000010216 <__libc_init_array>:
   10216:	1101                	addi	sp,sp,-32
   10218:	e822                	sd	s0,16(sp)
   1021a:	e04a                	sd	s2,0(sp)
   1021c:	6445                	lui	s0,0x11
   1021e:	6945                	lui	s2,0x11
   10220:	4d040793          	addi	a5,s0,1232 # 114d0 <__init_array_start>
   10224:	4d090913          	addi	s2,s2,1232 # 114d0 <__init_array_start>
   10228:	40f90933          	sub	s2,s2,a5
   1022c:	ec06                	sd	ra,24(sp)
   1022e:	e426                	sd	s1,8(sp)
   10230:	40395913          	srai	s2,s2,0x3
   10234:	00090b63          	beqz	s2,1024a <__libc_init_array+0x34>
   10238:	4d040413          	addi	s0,s0,1232
   1023c:	4481                	li	s1,0
   1023e:	601c                	ld	a5,0(s0)
   10240:	0485                	addi	s1,s1,1
   10242:	0421                	addi	s0,s0,8
   10244:	9782                	jalr	a5
   10246:	fe991ce3          	bne	s2,s1,1023e <__libc_init_array+0x28>
   1024a:	6445                	lui	s0,0x11
   1024c:	6945                	lui	s2,0x11
   1024e:	4d040793          	addi	a5,s0,1232 # 114d0 <__init_array_start>
   10252:	4e090913          	addi	s2,s2,1248 # 114e0 <__do_global_dtors_aux_fini_array_entry>
   10256:	40f90933          	sub	s2,s2,a5
   1025a:	40395913          	srai	s2,s2,0x3
   1025e:	00090b63          	beqz	s2,10274 <__libc_init_array+0x5e>
   10262:	4d040413          	addi	s0,s0,1232
   10266:	4481                	li	s1,0
   10268:	601c                	ld	a5,0(s0)
   1026a:	0485                	addi	s1,s1,1
   1026c:	0421                	addi	s0,s0,8
   1026e:	9782                	jalr	a5
   10270:	fe991ce3          	bne	s2,s1,10268 <__libc_init_array+0x52>
   10274:	60e2                	ld	ra,24(sp)
   10276:	6442                	ld	s0,16(sp)
   10278:	64a2                	ld	s1,8(sp)
   1027a:	6902                	ld	s2,0(sp)
   1027c:	6105                	addi	sp,sp,32
   1027e:	8082                	ret

0000000000010280 <memset>:
   10280:	433d                	li	t1,15
   10282:	872a                	mv	a4,a0
   10284:	02c37163          	bgeu	t1,a2,102a6 <memset+0x26>
   10288:	00f77793          	andi	a5,a4,15
   1028c:	e3c1                	bnez	a5,1030c <memset+0x8c>
   1028e:	e1bd                	bnez	a1,102f4 <memset+0x74>
   10290:	ff067693          	andi	a3,a2,-16
   10294:	8a3d                	andi	a2,a2,15
   10296:	96ba                	add	a3,a3,a4
   10298:	e30c                	sd	a1,0(a4)
   1029a:	e70c                	sd	a1,8(a4)
   1029c:	0741                	addi	a4,a4,16
   1029e:	fed76de3          	bltu	a4,a3,10298 <memset+0x18>
   102a2:	e211                	bnez	a2,102a6 <memset+0x26>
   102a4:	8082                	ret
   102a6:	40c306b3          	sub	a3,t1,a2
   102aa:	068a                	slli	a3,a3,0x2
   102ac:	00000297          	auipc	t0,0x0
   102b0:	9696                	add	a3,a3,t0
   102b2:	00a68067          	jr	10(a3)
   102b6:	00b70723          	sb	a1,14(a4)
   102ba:	00b706a3          	sb	a1,13(a4)
   102be:	00b70623          	sb	a1,12(a4)
   102c2:	00b705a3          	sb	a1,11(a4)
   102c6:	00b70523          	sb	a1,10(a4)
   102ca:	00b704a3          	sb	a1,9(a4)
   102ce:	00b70423          	sb	a1,8(a4)
   102d2:	00b703a3          	sb	a1,7(a4)
   102d6:	00b70323          	sb	a1,6(a4)
   102da:	00b702a3          	sb	a1,5(a4)
   102de:	00b70223          	sb	a1,4(a4)
   102e2:	00b701a3          	sb	a1,3(a4)
   102e6:	00b70123          	sb	a1,2(a4)
   102ea:	00b700a3          	sb	a1,1(a4)
   102ee:	00b70023          	sb	a1,0(a4)
   102f2:	8082                	ret
   102f4:	0ff5f593          	andi	a1,a1,255
   102f8:	00859693          	slli	a3,a1,0x8
   102fc:	8dd5                	or	a1,a1,a3
   102fe:	01059693          	slli	a3,a1,0x10
   10302:	8dd5                	or	a1,a1,a3
   10304:	02059693          	slli	a3,a1,0x20
   10308:	8dd5                	or	a1,a1,a3
   1030a:	b759                	j	10290 <memset+0x10>
   1030c:	00279693          	slli	a3,a5,0x2
   10310:	00000297          	auipc	t0,0x0
   10314:	9696                	add	a3,a3,t0
   10316:	8286                	mv	t0,ra
   10318:	fa2680e7          	jalr	-94(a3)
   1031c:	8096                	mv	ra,t0
   1031e:	17c1                	addi	a5,a5,-16
   10320:	8f1d                	sub	a4,a4,a5
   10322:	963e                	add	a2,a2,a5
   10324:	f8c371e3          	bgeu	t1,a2,102a6 <memset+0x26>
   10328:	b79d                	j	1028e <memset+0xe>

000000000001032a <__call_exitprocs>:
   1032a:	715d                	addi	sp,sp,-80
   1032c:	67c9                	lui	a5,0x12
   1032e:	f052                	sd	s4,32(sp)
   10330:	c307ba03          	ld	s4,-976(a5) # 11c30 <_global_impure_ptr>
   10334:	f84a                	sd	s2,48(sp)
   10336:	e486                	sd	ra,72(sp)
   10338:	1f8a3903          	ld	s2,504(s4)
   1033c:	e0a2                	sd	s0,64(sp)
   1033e:	fc26                	sd	s1,56(sp)
   10340:	f44e                	sd	s3,40(sp)
   10342:	ec56                	sd	s5,24(sp)
   10344:	e85a                	sd	s6,16(sp)
   10346:	e45e                	sd	s7,8(sp)
   10348:	e062                	sd	s8,0(sp)
   1034a:	02090863          	beqz	s2,1037a <__call_exitprocs+0x50>
   1034e:	8b2a                	mv	s6,a0
   10350:	8bae                	mv	s7,a1
   10352:	4a85                	li	s5,1
   10354:	59fd                	li	s3,-1
   10356:	00892483          	lw	s1,8(s2)
   1035a:	fff4841b          	addiw	s0,s1,-1
   1035e:	00044e63          	bltz	s0,1037a <__call_exitprocs+0x50>
   10362:	048e                	slli	s1,s1,0x3
   10364:	94ca                	add	s1,s1,s2
   10366:	020b8663          	beqz	s7,10392 <__call_exitprocs+0x68>
   1036a:	2084b783          	ld	a5,520(s1)
   1036e:	03778263          	beq	a5,s7,10392 <__call_exitprocs+0x68>
   10372:	347d                	addiw	s0,s0,-1
   10374:	14e1                	addi	s1,s1,-8
   10376:	ff3418e3          	bne	s0,s3,10366 <__call_exitprocs+0x3c>
   1037a:	60a6                	ld	ra,72(sp)
   1037c:	6406                	ld	s0,64(sp)
   1037e:	74e2                	ld	s1,56(sp)
   10380:	7942                	ld	s2,48(sp)
   10382:	79a2                	ld	s3,40(sp)
   10384:	7a02                	ld	s4,32(sp)
   10386:	6ae2                	ld	s5,24(sp)
   10388:	6b42                	ld	s6,16(sp)
   1038a:	6ba2                	ld	s7,8(sp)
   1038c:	6c02                	ld	s8,0(sp)
   1038e:	6161                	addi	sp,sp,80
   10390:	8082                	ret
   10392:	00892783          	lw	a5,8(s2)
   10396:	6498                	ld	a4,8(s1)
   10398:	37fd                	addiw	a5,a5,-1
   1039a:	04878463          	beq	a5,s0,103e2 <__call_exitprocs+0xb8>
   1039e:	0004b423          	sd	zero,8(s1)
   103a2:	db61                	beqz	a4,10372 <__call_exitprocs+0x48>
   103a4:	31092783          	lw	a5,784(s2)
   103a8:	008a96bb          	sllw	a3,s5,s0
   103ac:	00892c03          	lw	s8,8(s2)
   103b0:	8ff5                	and	a5,a5,a3
   103b2:	2781                	sext.w	a5,a5
   103b4:	ef89                	bnez	a5,103ce <__call_exitprocs+0xa4>
   103b6:	9702                	jalr	a4
   103b8:	00892703          	lw	a4,8(s2)
   103bc:	1f8a3783          	ld	a5,504(s4)
   103c0:	01871463          	bne	a4,s8,103c8 <__call_exitprocs+0x9e>
   103c4:	fb2787e3          	beq	a5,s2,10372 <__call_exitprocs+0x48>
   103c8:	dbcd                	beqz	a5,1037a <__call_exitprocs+0x50>
   103ca:	893e                	mv	s2,a5
   103cc:	b769                	j	10356 <__call_exitprocs+0x2c>
   103ce:	31492783          	lw	a5,788(s2)
   103d2:	1084b583          	ld	a1,264(s1)
   103d6:	8ff5                	and	a5,a5,a3
   103d8:	2781                	sext.w	a5,a5
   103da:	e799                	bnez	a5,103e8 <__call_exitprocs+0xbe>
   103dc:	855a                	mv	a0,s6
   103de:	9702                	jalr	a4
   103e0:	bfe1                	j	103b8 <__call_exitprocs+0x8e>
   103e2:	00892423          	sw	s0,8(s2)
   103e6:	bf75                	j	103a2 <__call_exitprocs+0x78>
   103e8:	852e                	mv	a0,a1
   103ea:	9702                	jalr	a4
   103ec:	b7f1                	j	103b8 <__call_exitprocs+0x8e>

00000000000103ee <__libc_fini_array>:
   103ee:	1101                	addi	sp,sp,-32
   103f0:	e822                	sd	s0,16(sp)
   103f2:	67c5                	lui	a5,0x11
   103f4:	6445                	lui	s0,0x11
   103f6:	4e040413          	addi	s0,s0,1248 # 114e0 <__do_global_dtors_aux_fini_array_entry>
   103fa:	4e878793          	addi	a5,a5,1256 # 114e8 <impure_data>
   103fe:	8f81                	sub	a5,a5,s0
   10400:	e426                	sd	s1,8(sp)
   10402:	ec06                	sd	ra,24(sp)
   10404:	4037d493          	srai	s1,a5,0x3
   10408:	c881                	beqz	s1,10418 <__libc_fini_array+0x2a>
   1040a:	17e1                	addi	a5,a5,-8
   1040c:	943e                	add	s0,s0,a5
   1040e:	601c                	ld	a5,0(s0)
   10410:	14fd                	addi	s1,s1,-1
   10412:	1461                	addi	s0,s0,-8
   10414:	9782                	jalr	a5
   10416:	fce5                	bnez	s1,1040e <__libc_fini_array+0x20>
   10418:	60e2                	ld	ra,24(sp)
   1041a:	6442                	ld	s0,16(sp)
   1041c:	64a2                	ld	s1,8(sp)
   1041e:	6105                	addi	sp,sp,32
   10420:	8082                	ret

0000000000010422 <atexit>:
   10422:	85aa                	mv	a1,a0
   10424:	4681                	li	a3,0
   10426:	4601                	li	a2,0
   10428:	4501                	li	a0,0
   1042a:	a009                	j	1042c <__register_exitproc>

000000000001042c <__register_exitproc>:
   1042c:	67c9                	lui	a5,0x12
   1042e:	c307b703          	ld	a4,-976(a5) # 11c30 <_global_impure_ptr>
   10432:	1f873783          	ld	a5,504(a4)
   10436:	c3b1                	beqz	a5,1047a <__register_exitproc+0x4e>
   10438:	4798                	lw	a4,8(a5)
   1043a:	487d                	li	a6,31
   1043c:	06e84263          	blt	a6,a4,104a0 <__register_exitproc+0x74>
   10440:	c505                	beqz	a0,10468 <__register_exitproc+0x3c>
   10442:	00371813          	slli	a6,a4,0x3
   10446:	983e                	add	a6,a6,a5
   10448:	10c83823          	sd	a2,272(a6)
   1044c:	3107a883          	lw	a7,784(a5)
   10450:	4605                	li	a2,1
   10452:	00e6163b          	sllw	a2,a2,a4
   10456:	00c8e8b3          	or	a7,a7,a2
   1045a:	3117a823          	sw	a7,784(a5)
   1045e:	20d83823          	sd	a3,528(a6)
   10462:	4689                	li	a3,2
   10464:	02d50063          	beq	a0,a3,10484 <__register_exitproc+0x58>
   10468:	00270693          	addi	a3,a4,2
   1046c:	068e                	slli	a3,a3,0x3
   1046e:	2705                	addiw	a4,a4,1
   10470:	c798                	sw	a4,8(a5)
   10472:	97b6                	add	a5,a5,a3
   10474:	e38c                	sd	a1,0(a5)
   10476:	4501                	li	a0,0
   10478:	8082                	ret
   1047a:	20070793          	addi	a5,a4,512
   1047e:	1ef73c23          	sd	a5,504(a4)
   10482:	bf5d                	j	10438 <__register_exitproc+0xc>
   10484:	3147a683          	lw	a3,788(a5)
   10488:	4501                	li	a0,0
   1048a:	8e55                	or	a2,a2,a3
   1048c:	00270693          	addi	a3,a4,2
   10490:	068e                	slli	a3,a3,0x3
   10492:	2705                	addiw	a4,a4,1
   10494:	30c7aa23          	sw	a2,788(a5)
   10498:	c798                	sw	a4,8(a5)
   1049a:	97b6                	add	a5,a5,a3
   1049c:	e38c                	sd	a1,0(a5)
   1049e:	8082                	ret
   104a0:	557d                	li	a0,-1
   104a2:	8082                	ret

00000000000104a4 <_exit>:
   104a4:	05d00893          	li	a7,93
   104a8:	00000073          	ecall
   104ac:	00054363          	bltz	a0,104b2 <_exit+0xe>
   104b0:	a001                	j	104b0 <_exit+0xc>
   104b2:	1141                	addi	sp,sp,-16
   104b4:	e022                	sd	s0,0(sp)
   104b6:	842a                	mv	s0,a0
   104b8:	e406                	sd	ra,8(sp)
   104ba:	4080043b          	negw	s0,s0
   104be:	008000ef          	jal	ra,104c6 <__errno>
   104c2:	c100                	sw	s0,0(a0)
   104c4:	a001                	j	104c4 <_exit+0x20>

00000000000104c6 <__errno>:
   104c6:	8101b503          	ld	a0,-2032(gp) # 11c40 <_impure_ptr>
   104ca:	8082                	ret
