# S07: 프로젝트 전용 MSVC용 Clang 도구 공급

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11, R08
- 담당/수정 범위: root — 도구 복사·native 명령·증거·문서 통합; soak_recovery_audit — /tmp 원pins/시험 계획 독립 준비
- 선행 작업: 기존 LLVM 도구 pin과 프로젝트 전용 MSVC std metadata 인수 완료

## 사용자 결과

Windows 개발 환경에서 전역 설치 없이 MSVC용 compiler/linker 실행 가능 여부를 확인한다. 이 작업만으로 일반 사용 릴리스 또는 MSVC 앱 빌드 완료를 뜻하지 않는다.

## 계약과 범위

기존 clang-23.exe와 ld.lld.exe를 fresh 프로젝트 경로의 clang-cl.exe/lld-link.exe로 복사한다. 원 bin/DLL/resource는 그대로 유지한다. 도구에만 child-local PATH와 resource 경로를 지정한다. alias 추가 디스크 비용 5,303,296B이며 새 앱 dependency/lock 변경 없다. 도구 runtime 비용은 측정 전 미확인이다. 변경 범위 .local-tools/windows/msvc-supply-20261006/clang-tools 및 docs/validation/evidence/windows-msvc-clang-tools-*와 본 기록. SDK/CRT 다운로드·관리자 변경·앱 링크는 별도다.

## 인수 조건

원 및 복사한 실행 파일 SHA/bytes 대조, native 버전과 COFF linker 도움말, SDK 없이 합성 C 파일 객체 생성. AMD64 COFF 헤더/section table bounded 검토, 원 도구 보존, 가상 화면 원 Job 자연 종료 및 입력 화면 불변. 실패 raw도 보존한다. full MSVC 앱/최소 OS 지원은 미검증으로 남긴다.

## 검증과 handoff

독립 준비 계획과 원pins를 /tmp에 작성했다. 실제 복사/native 실행 전이다. SDK/CRT import library 및 native Rust MSVC 앱 링크는 별도 후속이다.


실제 `python3 scripts/windows_msvc_clang_tools_collect.py --pins /tmp/dlpad-clang-cl-supply-pins.json --base .local-tools/windows/msvc-supply-20261006/clang-driver-native --output docs/validation/evidence/windows-msvc-clang-tools-20261006-continue`(모든 경로는 실제 absolute로 실행) exit0. clang23.1.2/Target x86_64-pc-windows-msvc, lld COFF 도움말, MSVCtarget 합성C object 생성 모두 private 원Job자연0/input불변. 원13pins·alias2·notice동일bytes·AMD64 COFF669B/5sections SHA1f90fbce…를 root/독립담당이 대조했다. [결과](../validation/evidence/windows-msvc-clang-tools-20261006-continue/result.json), [주 담당](../validation/evidence/windows-msvc-clang-tools-20261006-continue/root-completion-review.json), [독립](../validation/evidence/windows-msvc-clang-tools-20261006-continue/independent-review.json). scoped object smoke 완료이며 SDK/CRT·Rust MSVC 앱 링크·도구 runtime비용/notice완비는 미완료다.
