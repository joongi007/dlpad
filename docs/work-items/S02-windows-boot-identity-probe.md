# S02: Windows native boot identity 후보 실험

- 상태: completed (후보 실험만; 제품 권한 미채택)
- 분류: prototype
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: metrics_recovery — scripts/windows_boot_identity_probe.rs 및 이 문서만
- 선행 작업: Windows 보류 쿠키 회수 설계; root 독립 native GNU 컴파일·hidden CLI 실행

## 사용자 결과

이 실험은 Windows 이전 부팅 회수의 증거 후보를 조사한다. 제품 recovery나 삭제 권한을 구현하지 않는다. Linux/WSL boot UUID를 재사용하지 않는다. API 채택은 Proposed이며 실제 재부팅은 별도 게이트다.

## 공식 지원 경계

[Microsoft NtQuerySystemInformation](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation)은 향후 변경 또는 미지원 가능성을 명시한다. [Microsoft SDK winternl.h](https://github.com/microsoft/win32metadata/blob/main/generation/WinSDK/RecompiledIdlHeaders/um/winternl.h)는 internal 구조 변경 경고와 dynamic lookup 권고를 제공한다. 읽은 공개 header와 문서에서 SYSTEM_BOOT_ENVIRONMENT_INFORMATION의 BootIdentifier layout 및 재부팅별 유일성 계약을 확보하지 못했다. 따라서 class90/32B/첫16B 해석은 실험 가정이며 공식 지원 확정으로 보고하지 않는다. BCD boot entry GUID 역시 현재 부팅 실행 identity의 대체물이 아니다.

## 계약과 범위

의존성 없는 standalone Rust GNU 프로그램이다. 이미 로드된 ntdll의 NtQuerySystemInformation만 GetModuleHandleW/GetProcAddress로 조회한다. 고정 class90, 8B 정렬된 32B 저장소를 세 번 질의하고 NTSTATUS/ReturnLength/비영 GUID 후보 및 동일 process 일치를 기록한다. 반환 길이가 정확32가 아니거나 0 GUID/반복 불일치면 fail-closed한다. 미지원/권한 오류는 Unavailable이며 크기를 늘려 임의 layout을 해석하지 않는다. 원시 GUID는 Windows GUID 문자열이 아닌 byte-order 후보 hex다.

모든 출력은 authority_eligible=false다. 같은 process 반복 및 다른 process 일치는 후보 안정성만 보이며 reboot/hibernate/Fast Startup 의미를 입증하지 않는다. 쿠키/경로/계정/보안/전원 변경, GUI, 재부팅은 없다. workspace/Cargo/schema와 제품 코드도 변경하지 않는다. native syscall에 사용자 공간 강제 timeout이 없으므로 root 소유 hidden CLI host Job의 유한 기한으로 전체 process를 제한한다.

## 인수 조건

root가 native rustc GNU로 임시 실행파일을 만들고 hidden CLI host로 서로 다른 process에서 세 번 실행한다. 각 실행 exit/status/length/nonzero/반복 일치를 확인하고 process 간 후보도 비교한다. stdout 전체는 16KiB 상한으로 저장한다. source/tool SHA256와 bytes, toolchain, 원 argv, elapsed time 및 호스트 deadline을 원시 evidence에 기록한다. query 시간은 API 질의·출력 포함한 제한 범위이며 제품 전체 비용을 대표하지 않는다. malformed 및 unsupported 결과는 삭제 권한을 부여하지 않는다.

## 검증과 handoff

공식 자료 browsing과 standalone source 준비만 수행했다. 실제 Windows 컴파일·실행 및 reboot는 root 검증 전이다. `python3 scripts/check_docs.py` 실제469개 PASS. rustfmt는 PATH에 없어 명령 exit127로 미실행이며 root native 도구로 포맷·컴파일 확인이 필요하다. source SHA256은 86f8d12893c9cd58dba3825e2f0e9c48578afbb78ad5e2fdc4ce69d7e20ea867이다. 출처·layout 지원과 reboot별 변경이 확인되지 않으면 이전 부팅 회수는 Unavailable로 유지한다.

## Root 실제 native 실험

프로젝트 고정 rustfmt1.94로 standalone source를 포맷했다. sourceSHA5f8eed16e6d1ef51b65a6b48d5700f380cd2091b80e51468947510639f9b85bb. Windows GNU rustc native compile exit0, exe2071040B/SHAf8e1169b1d961a1218e0d5b459dca71b0c5b364bb26a0769d3990f3ab26576b5. first의 query3회는 host 최소 인자 검사 usage/exit2로 프로그램이 실행되지 않았고 보존했다. 고정 --probe 인자를 추가한 second에서 서로 다른 PID24392/16036/19280의 실제 query각3회, status0/returned32/동일비영candidate 및 exit0/private desktop/inputdesktopunchanged를 원시 로그로 대조했다. source변경없음.

증거: [first](../validation/evidence/windows-boot-candidate-native-first/root-review.json), [second](../validation/evidence/windows-boot-candidate-native-second/root-review.json). 질의+출력3회 시간45100~61700ns는 좁은 프로그램 측정이며 제품 전체 비용이 아니다. exact argv/tool SHA는 각compile/query JSON과 first root-review에 보존했다.

같은boot 안정성만 확인됐다. 지원layout/재부팅별유일성/FastStartup는 미확보이므로 authority_eligible=false를 유지하고 제품 이전부팅 회수는 채택하지 않는다. Windows sameboot 독립Job/guardian/journal/receipt 연결은 여전히 필수 구현이다.
