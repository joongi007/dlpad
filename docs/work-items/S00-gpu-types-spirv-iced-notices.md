# S00: GPU types·SPIR-V·Iced WGPU notice

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: root는 windows_candidate_package.py·upstreammanifest·notice3원문·본 문서, diagnostic_cache는 /tmp 수집·검토
- 선행 작업: GPU exactmapping23source/33package와 missing50

## 사용자 결과

gpu-descriptor-types0.2.0, spirv0.3.0+sdk1.3.268.0, iced_wgpu0.14.0의 exact revision notice를 후보에 연결한다.

## 계약과 범위

각 Cargo 이름/버전/license/VCS revision/path/원문SHA 검증을 유지하고 Iced의 기존 원notice를 재사용한다. source26<=32/package36<=64, deps/Cargo/Rust/제품 실행비용0, 신규 원notice12,790B. Apache 원문을 SPIR-V embedded registry 저작권 자료 전체 판정으로 확대하지 않는다. coverage_complete/release_ready=false 유지.

## 인수 조건

positive3/잘못된commit·repo·path·license·grant 거절, 원source exactbytesSHA 보존. 새2ZIP 동일SHA/inventory 독립검증에서 missing 감소를 실측한다.

## 검증과 handoff

/tmp 초안은 root 검토하며 전체 P0/법적완비는 미완료다.

root원cachemanifest/VCS/notice원문SHA검토후positive3/negative21(revision/repo/path/emptyset·sourceSHA/bytes/URL) PASS다. 정확26source/36package이며원Iced자료재사용, 신규제품비용0. 첫새후보실측진행중이다.

최종2ZIP SHA20fb39443a8e6080a0e2ec302a2ba0310848eb91a71c0a2da392f7050e115c3c 동일/12,646,828B/858entries/missing47, root bounded독립review PASS다. 이전36전체mapping도positive로대조했다. source26/package36/coverage_complete=false이며실제linkedWindowsreachability·embeddedlicense·release_ready는미완료다.
