# S07: 고정 Rust MSVC 표준 라이브러리 공급

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 프로젝트 전용 sysroot·공급 실행·증거·문서; diagnostic_cache — /tmp 공식 channel/sidecar metadata; soak_recovery_audit — 공급 후 독립 증거 검토
- 선행 작업: 공식 Rust1.94.0 channel과 정확 MSVC std sidecar SHA 일치. GNU 원 도구를 유지한다.

## 사용자 결과

Windows MSVC 검증에 필요한 정확 Rust std를 프로젝트 전용 새 경로에 준비하고, 현재 native compiler가 해당 target std를 읽는지 확인한다.

## 계약과 범위

공식 2026-03-05 release의 x86_64-pc-windows-msvc std를 크기·SHA 상한으로 다운로드한다. archive inventory·경로·link·개별/전체 크기를 검증하고 원 component의 target tree와 부속 notice를 create-new 경로에만 추출한다. install.sh 실행·기존 GNU/시스템 도구 덮어쓰기·관리자 변경은 없다. 라이선스/manifest와 정확 비용을 보존한다. native rustc target-libdir 조회와 metadata-only std 사용 시험은 실제 MSVC 링크/앱 실행 인수와 구분한다. Cargo/lock/제품 schema 변경 없음.

## 인수 조건

정확 archive22,323,988B/SHA f8e2b43e…·sidecar/channel 일치·bounded archive64entries/전체114,076,151B inventory·safe 새 sysroot·원GNU 도구 SHA 불변·native 두 명령 소유 Job/private/input guards. SDK/CRT/linker 미공급이면 native MSVC 전체 빌드를 완료로 보고하지 않는다. 실패/부분 산출물을 자동 삭제하지 않는다.

## 검증과 handoff

고정 archive 다운로드는 실제 PASS(6.17초)이며 inventory를 읽었다. 추출/native 명령은 다음 단계다. 개발 도구 notice와 앱 배포 SBOM을 혼동하지 않는다. Microsoft VS17/VS16 manifest의 선언/응답 SHA 불일치는 별도 조사 중이며 이 표준 라이브러리 공급은 그 실패 경로에 의존하지 않는다.


최종 범위 완료: [주 담당 검토](../validation/evidence/windows-msvc-std-native-20261006-continue/root-completion-review.json)와 [독립 검토](../validation/evidence/windows-msvc-std-native-20261006-continue/independent-review.json) PASS. 실제 source576/원GNU8tool SHA·size 불변·archive22,323,988B 및 원52파일114,034,067B를 대조했다. native rustc의 정확 MSVC target-libdir 조회와 std::Instant/PathBuf metadata 생성2,194B를 확인했다. metadata는 PE나 MSVC 앱 링크가 아니다. install.sh 실행/기존도구 덮어쓰기0; SDK/CRT/링커와 실제MSVC 전체검증은 후속이다. archive MIT/Apache/COPYRIGHT 원bytes를 보존했으며 referenced COPYRIGHT-library.html는 이 component에 없으므로 전체 법적 재배포 검토 완료를 주장하지 않는다.
