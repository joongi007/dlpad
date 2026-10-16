# S04: 고정 FFmpeg TAR 형식·XZ 메모리 실측

- 상태: in-progress
- 분류: review
- 우선순위: P0
- 요구사항: R05, R11
- 담당/수정 범위: root — 이 문서와 docs/validation/evidence/ffmpeg-tar-header-inspection/; diagnostic_cache — 별도 bounded_tar.rs/자체 작업 계약
- 선행 작업: 고정 archive와 component 공급 감사

## 사용자 결과

관리 FFmpeg 설치의 parser/압축 메모리 계약을 실제 원 archive 형식에 맞춘다.

## 계약과 범위

137863452byte/SHA2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a의 원파일을 읽기전용 확인하고 Python 표준 LZMA decoder의 64MiB 메모리 상한·출력4096byte 상한으로 첫 TAR header를 관측한다. 전체 추출/도구실행/원본수정/네트워크 없음. 제품 decoder 채택 근거와 실측은 별도이며 새 Cargo/schema는 root 후속 조정한다.

## 인수 조건

원size/SHA 일치, header magic/type/name과 XZ 메모리 상한 성공 여부 및 원시 결과를 기록한다. 오류는 형식 성공으로 표시하지 않는다.

## 검증과 handoff

실측 시작.

고정원archive137863452byte/SHA일치를stream확인했고PythonLZMADecompressor(memlimit=64MiB)의첫4096byte출력은0.95초에성공했다. 실제첫directory/regular header는GNU magic `ustar  ` 계열(257:265 hex7573746172202000)이며POSIX magic만받는parser로는실제공급을읽을수없다. [원시 관측](../validation/evidence/ffmpeg-tar-header-inspection/result.json). first_blocks에는단순512byte관측payloadblock도포함하므로전체inventory또는모든block을header로검증했다는뜻이아니다. root가reader담당에게정확GNUmagic과GNUextension거절/prefix오인주의를전달했다. 이실험은제품Rustdecoder의peak/전체검증을대신하지않는다.
