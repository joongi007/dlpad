# S07: MSVC 실험의 GNU host 빌드 스크립트 링크

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — wrapper 인터페이스·native실행·Cargo설정·증거/문서; diagnostic_cache — /tmp C# wrapper초안; soak_recovery_audit — 독립 source 검토
- 선행 작업: 전체 MSVC 실험의 locked targetcrate 공급. 현재 quote HOSTGNU buildscript의 gcc_eh/gcc 링크가 실패하며 MSVCtarget compile은 아직 인수 전.

## 사용자 결과

기존 Windows GNU-host rustc로 빌드 스크립트/procmacro를 만들고 실제 앱은 MSVCtarget으로 빌드할 수 있게 두 링크 설정을 분리한다.

## 계약과 범위

프로젝트 전용 native RUSTC_WRAPPER를 사용해 HOSTGNU 컴파일에만 -C link-self-contained=yes를 추가한다. exact MSVCtarget에는 해당GNUflag를 넣지 않는다. 고정 rootRustc/clippy-driver만 argv로 실행하고 Winquote/수량/명령줄길이 상한과 original outerJob 보호를 유지한다. shell/PID기반kill/전역tool교체 없음. 변경 범위 scripts C# 및 compilerhelper collector, 실험설정, fresh evidence/본문. Cargo dependency/lock/앱runtime변경은 없다. helper는 developer compiler 호출마다 CLR 프로세스를 추가하므로 실제bytes 및 미측정 startup/runtime비용을 기록한다.

## 인수 조건

잘못된compiler/경로/targetargv 거절, HOSTGNU 합성compile/실행 또는 실제원buildscript성공, MSVCtarget metadata/전체실험에서GNUflag없음, 원Job자연0/private/input/source/toolpin보존. 원실패를보존하고 root+독립 검토. productlauncher/최종stack/전체P0 완료가 아니다.

## 검증과 handoff

초안 작성 전이며 root가 actual source/통합/실행을 조정한다. 기존 GNU release artifacts·SDK/Ruststd·Cargo.lock은 변경하지 않는다.


실제6명령 compilewrapper/compilepolicy/policy8/GNU-host compile/합성GNU실행/MSVCtarget metadata compile exit0 및 원Job자연0/private종료/input불변/source576·compiler원pins보존. wrapper7,680B/SHAab1f472b…; 선택분기trace actualHOST self-contained와 MSVC target unchanged 확인. MSVC binarycrate rmeta는0B이며 nonemptylibrary metadata로표현하지 않는다. 원Job감독이있는개발용 nativewrapper로범위완료, standalonetimeout/responsefile/조상dirrace/제품런처/cleanOS는보호인수외다. root는 입력source와target-policy/actual원로그를읽었고 독립담당도[원결과](../validation/evidence/windows-msvc-host-wrapper-20261006-continue/result.json)와[독립보고](../validation/evidence/windows-msvc-host-wrapper-20261006-continue/independent-review.json)를대조했다. helper는compile호출마다CLRprocess추가이므로startup/RSS예산은미측정개발비용이며제품runtime에는추가하지않는다. 전체MSVCworkspace는이helper를적용해별도진행중이다.

실제Cargo 통합에서는 .NET의 기본stdout상속이 Cargo capturepipe로 전달되지 않아 rustc -vV hostline이비어 compilerquery실패했다. 원source/policy/6actual PASS는당시제한된scope로보존하며 일반Cargo적합성완료로사용하지않는다. 새wrapper는stdout/stderr각rawbyte펌핑(64KiBbuffer/각8MiB상한)과 원5초join/오류125, Cargoquery stdin명시EOF로보완했다. 오류시원reader를닫아child출력backpressure를완화한다. STDINcompile는지원하지않고거절한다. 실제-vV hostline을포함한새7native명령과 Cargo통합을검증해야범위완료다.

새 [stdout suite](../validation/evidence/windows-msvc-host-wrapper-stdout-20261006-continue/result.json)의 실제7명령 및 root/독립 raw·compiler/helper pins·source576 대조를 완료했다. 이어 [전체 MSVC workspace](../validation/evidence/windows-msvc-workspace-stdout-20261006-continue/result.json)의 fmt/clippy/test/release 네 명령과 테스트758passed/0failed/13ignored가 실제 통과해 Cargo capturepipe 통합도 확인했다. 현재 helper9,216B/SHAeaeb11367a018baf56ae9647c0c34eb3cae5b2ea0e0d885cdab03724766050f9이며 앱runtime/lock 변경은 없다. 출력초과·destination pump 실패의 native 주입 시험, standalone 취소, Windows ambient 환경 제거 및 전체 제품P0는 이 완료 범위에 포함하지 않는다.
